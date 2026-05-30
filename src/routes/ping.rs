use actix_web::{Responder, get};

#[get("/ping")]
pub(crate) async fn ping() -> impl Responder {
    String::from("pong")
}
