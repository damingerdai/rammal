use crate::state::AppState;
use actix_web::{Error, HttpRequest, HttpResponse, web};

pub(crate) async fn auth_proxy(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
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
