use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use actix_web::{Error, HttpRequest, HttpResponse, web};
use futures_util::StreamExt;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::Receiver;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::state::AppState;

fn cache_path_for_request(cache_dir: &str, path: &str, req: &HttpRequest) -> Option<PathBuf> {
    if req.method() != actix_web::http::Method::GET
        || !req.query_string().is_empty()
        || req.headers().contains_key("range")
        || req
            .headers()
            .keys()
            .any(|name| name.as_str().starts_with("if-"))
    {
        return None;
    }
    let (repository, digest) = path.rsplit_once("/blobs/")?;
    if repository.split('/').any(|part| {
        part.is_empty()
            || part == "."
            || part == ".."
            || !part
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    }) {
        return None;
    }
    let hash = digest.strip_prefix("sha256:")?;
    if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    Some(
        Path::new(cache_dir)
            .join(repository)
            .join("blobs")
            .join(digest),
    )
}

enum CacheMessage {
    Chunk(web::Bytes),
    Complete,
    Failed,
}

async fn write_cache(path: PathBuf, mut rx: Receiver<CacheMessage>) {
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    let Some(parent) = path.parent() else { return };
    if tokio::fs::create_dir_all(parent).await.is_err() {
        return;
    }
    let temporary = path.with_extension(format!(
        "{}.{}.part",
        std::process::id(),
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let Ok(mut file) = File::options()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .await
    else {
        return;
    };
    let result: std::io::Result<()> = async {
        while let Some(message) = rx.recv().await {
            match message {
                CacheMessage::Chunk(bytes) => file.write_all(&bytes).await?,
                CacheMessage::Complete => {
                    file.flush().await?;
                    tokio::fs::rename(&temporary, &path).await?;
                    return Ok(());
                }
                CacheMessage::Failed => break,
            }
        }
        Err(std::io::Error::other("Incomplete blob download"))
    }
    .await;
    drop(file);
    if result.is_err() {
        let _ = tokio::fs::remove_file(&temporary).await;
    }
}

pub(crate) async fn proxy_handler(
    req: HttpRequest,
    payload: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let client = &state.client;
    // 构建上游url
    let path = req.match_info().query("path");
    let query_string = req.query_string();

    let cache_path = cache_path_for_request(&state.cache_dir, path, &req);

    // Only completed downloads are published at this path; .part files are ignored.
    if let Some(cache_path) = &cache_path {
        if let Ok(file) = File::open(cache_path).await {
            if let Ok(metadata) = file.metadata().await {
                if metadata.is_file() {
                    let digest = path.rsplit('/').next().unwrap();
                    let stream = FramedRead::new(file, BytesCodec::new())
                        .map(|result| result.map(|bytes| bytes.freeze()));
                    return Ok(HttpResponse::Ok()
                        .content_type("application/octet-stream")
                        .insert_header(("content-length", metadata.len().to_string()))
                        .insert_header(("docker-content-digest", digest))
                        .insert_header(("etag", format!("\"{digest}\"")))
                        .streaming(stream));
                }
            }
        }
    }

    let upstream = &state.upstream;

    let upstream_url = if query_string.is_empty() {
        format!("{}/v2/{}", upstream, path)
    } else {
        format!("{}/v2/{}?{}", upstream, path, query_string)
    };

    println!("Forwarding request to: {}", upstream_url);

    let forwarded_req = client
        .request_from(upstream_url.as_str(), req.head())
        .no_decompress();

    let res = forwarded_req
        .send_stream(payload)
        .await
        .map_err(actix_web::error::ErrorBadGateway)?;

    let status = res.status();
    let mut client_resp = HttpResponse::build(status);

    for (header_name, header_value) in res.headers().iter() {
        if header_name == "www-authenticate" {
            let auth_val = header_value
                .to_str()
                .unwrap_or("")
                .replace("https://auth.docker.io", "/auth");
            client_resp.insert_header((header_name.clone(), auth_val));
        } else {
            client_resp.insert_header((header_name.clone(), header_value.clone()));
        }
    }

    if let Some(cache_path) = cache_path.filter(|_| {
        status == actix_web::http::StatusCode::OK
            && !res.headers().contains_key("content-encoding")
            && !res.headers().contains_key("content-range")
    }) {
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        actix_web::rt::spawn(write_cache(cache_path, rx));
        let output_stream = futures_util::stream::unfold((res, tx), |(mut res, tx)| async move {
            match res.next().await {
                Some(Ok(bytes)) => {
                    // Async backpressure keeps memory bounded without blocking a worker.
                    let _ = tx.send(CacheMessage::Chunk(bytes.clone())).await;
                    Some((Ok::<_, Error>(bytes), (res, tx)))
                }
                Some(Err(error)) => {
                    let _ = tx.send(CacheMessage::Failed).await;
                    Some((Err(actix_web::error::ErrorBadGateway(error)), (res, tx)))
                }
                None => {
                    let _ = tx.send(CacheMessage::Complete).await;
                    None
                }
            }
        });
        return Ok(client_resp.streaming(output_stream));
    }

    Ok(client_resp.streaming(res))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;

    #[test]
    fn caches_only_complete_blob_get_requests_with_safe_paths() {
        let path = format!("library/alpine/blobs/sha256:{}", "a".repeat(64));
        let req = TestRequest::default().to_http_request();
        assert!(cache_path_for_request("cache", &path, &req).is_some());
        for unsafe_path in [
            format!("../{path}"),
            format!("/{path}"),
            path.replace("sha256:", "invalid:"),
        ] {
            assert!(cache_path_for_request("cache", &unsafe_path, &req).is_none());
        }
        for req in [
            TestRequest::default()
                .method(actix_web::http::Method::HEAD)
                .to_http_request(),
            TestRequest::default()
                .insert_header(("range", "bytes=0-10"))
                .to_http_request(),
            TestRequest::default()
                .insert_header(("if-none-match", "etag"))
                .to_http_request(),
            TestRequest::with_uri("/?foo=bar").to_http_request(),
        ] {
            assert!(cache_path_for_request("cache", &path, &req).is_none());
        }
    }

    #[actix_web::test]
    async fn serves_completed_blob_without_contacting_upstream() {
        let directory = std::env::temp_dir().join(format!(
            "rammal-cache-hit-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let digest = format!("sha256:{}", "a".repeat(64));
        let path = format!("library/alpine/blobs/{digest}");
        let cache_path = directory.join(&path);
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        tx.send(CacheMessage::Chunk(web::Bytes::from_static(b"blob data")))
            .await
            .unwrap();
        tx.send(CacheMessage::Complete).await.unwrap();
        drop(tx);
        write_cache(cache_path.clone(), rx).await;

        let app = actix_web::test::init_service(
            actix_web::App::new()
                .app_data(web::Data::new(AppState {
                    client: awc::Client::default(),
                    // Any attempt to use upstream must fail, without requiring network access.
                    upstream: "unsupported://upstream".into(),
                    cache_dir: directory.to_str().unwrap().into(),
                }))
                .route("/v2/{path:.*}", web::to(proxy_handler)),
        )
        .await;
        let req = TestRequest::get().uri(&format!("/v2/{path}")).to_request();
        let response = actix_web::test::call_service(&app, req).await;
        assert_eq!(response.status(), actix_web::http::StatusCode::OK);
        assert_eq!(response.headers().get("content-length").unwrap(), "9");
        assert_eq!(
            response.headers().get("docker-content-digest").unwrap(),
            digest.as_str()
        );
        assert_eq!(
            actix_web::test::read_body(response).await,
            web::Bytes::from_static(b"blob data")
        );

        // A temporary download must never count as a cache hit.
        tokio::fs::rename(&cache_path, cache_path.with_extension("part"))
            .await
            .unwrap();
        let req = TestRequest::get().uri(&format!("/v2/{path}")).to_request();
        let response = actix_web::test::call_service(&app, req).await;
        assert_eq!(response.status(), actix_web::http::StatusCode::BAD_GATEWAY);
        tokio::fs::remove_dir_all(directory).await.unwrap();
    }

    #[actix_web::test]
    async fn publishes_only_completed_downloads_and_cleans_partial_files() {
        let directory = std::env::temp_dir().join(format!(
            "rammal-cache-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for (name, ending) in [
            ("complete", Some(CacheMessage::Complete)),
            ("failed", Some(CacheMessage::Failed)),
            ("disconnected", None),
        ] {
            let path = directory.join(name);
            let (tx, rx) = tokio::sync::mpsc::channel(8);
            tx.send(CacheMessage::Chunk(web::Bytes::from_static(b"blob data")))
                .await
                .unwrap();
            if let Some(ending) = ending {
                tx.send(ending).await.unwrap();
            }
            drop(tx);
            write_cache(path.clone(), rx).await;
            if name == "complete" {
                assert_eq!(tokio::fs::read(path).await.unwrap(), b"blob data");
            } else {
                assert!(!path.exists());
            }
        }
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);
        tokio::fs::remove_dir_all(directory).await.unwrap();
    }
}
