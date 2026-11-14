use std::time::Duration;

use actix_web::{App, HttpServer, web};
use awc::{Client, Connector};
use state::AppState;

use settings::Settings;

mod routes;
mod settings;
mod state;

const UPSTREAM: &str = "https://registry-1.docker.io";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = Settings::load().expect("Failed to load configuration");

    let host = settings.server.host.clone();
    let port = settings.server.port;
    println!("Actix Docker Proxy is running {host}:{port}");
    HttpServer::new(move || {
        let connector =
            Connector::new().timeout(Duration::from_secs(settings.proxy.connect_timeout));
        let client = Client::builder()
            .connector(connector)
            .timeout(Duration::from_secs(settings.proxy.request_timeout))
            .finish();
        let app_state = web::Data::new(AppState { client });

        App::new()
            .app_data(app_state.clone())
            .service(routes::ping)
            .service(routes::version)
            .route("/auth/token", web::get().to(routes::auth_proxy))
            .route("/v2/{path:.*}", web::to(routes::proxy_handler))
    })
    .bind((host, port))?
    .run()
    .await
}
