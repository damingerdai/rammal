use std::time::Duration;

use actix_web::{App, Error, HttpRequest, HttpResponse, HttpServer, Responder, get, web};
use awc::{Client, Connector};

const CACHE_DIR: &str = "./docker_cache";
const UPSTREAM: &str = "https://registry-1.docker.io";

struct AppState {
    client: Client,
}

#[get("/")]
async fn index() -> impl Responder {
    // add code here
    "hello world"
}

#[get("/ping")]
async fn ping() -> impl Responder {
    "ping"
}

async fn proxy_handler(
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

async fn auth_proxy(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse, Error> {
    let query = req.query_string();
    let auth_url = format!("https://auth.docker.io/token?{}", query);
    let res = state
        .client
        .get(auth_url)
        .send()
        .await
        .map_err(actix_web::error::ErrorBadGateway)?;

    let mut client_resp = HttpResponse::build(res.status());

    Ok(client_resp.streaming(res))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Actix Docker Proxy is running 0.0.0.0:8000");
    HttpServer::new(|| {
        //let client_data = Client::default();
        let connector = Connector::new().timeout(Duration::from_secs(30));
        let client = Client::builder()
            .connector(connector)
            .timeout(Duration::from_secs(60))
            .finish();
        let app_data = web::Data::new(AppState { client });
        App::new()
            .app_data(app_data)
            .service(index)
            .service(ping)
            .route("/auth/token", web::get().to(auth_proxy))
            .route("/v2/{path:.*}", web::to(proxy_handler))
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
