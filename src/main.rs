use std::time::Duration;

use actix_web::{App, HttpServer, web};
use awc::{Client, Connector};
use state::AppState;

mod routes;
mod state;

const CACHE_DIR: &str = "./docker_cache";
const UPSTREAM: &str = "https://registry-1.docker.io";

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
            .service(routes::ping)
            .route("/auth/token", web::get().to(routes::auth_proxy))
            .route("/v2/{path:.*}", web::to(routes::proxy_handler))
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
