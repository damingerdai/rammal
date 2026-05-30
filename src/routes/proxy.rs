use crate::state::AppState;
use actix_web::{Error, HttpRequest, HttpResponse, web};

const UPSTREAM: &str = "https://registry-1.docker.io";

pub(crate) async fn proxy_handler(
    req: HttpRequest,
    payload: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let client = &state.client;
    // 构建上游url
    let path = req.match_info().query("path");
    let query_string = req.query_string();
    let upstream_url = if query_string.is_empty() {
        format!("{}/v2/{}", UPSTREAM, path)
    } else {
        format!("{}/v2/{}?{}", UPSTREAM, path, query_string)
    };

    println!("正在转发请求到: {}", upstream_url);

    let forwarded_req = client
        .request_from(upstream_url.as_str(), req.head())
        .no_decompress(); // 保持原样转发

    let res = forwarded_req
        .send_stream(payload)
        .await
        .map_err(actix_web::error::ErrorBadGateway)?;

    let mut client_resp = HttpResponse::build(res.status());

    for (header_name, header_value) in res.headers().iter() {
        if header_name == "www-authenticate" {
            // 将 auth.docker.io 替换为本代理地址，欺骗客户端继续走代理
            let auth_val = header_value
                .to_str()
                .unwrap_or("")
                .replace("https://auth.docker.io", "/auth");
            client_resp.insert_header((header_name.clone(), auth_val));
        } else {
            client_resp.insert_header((header_name.clone(), header_value.clone()));
        }
    }

    Ok(client_resp.streaming(res))
}
