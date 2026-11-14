use actix_web::{Responder, get};
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct VersionInfo {
    version: &'static str,
    commit: &'static str,
    commit_date: &'static str,
    build_date: &'static str,
    rustc_version: &'static str,
    target: &'static str,
}

#[get("/version")]
pub(crate) async fn version() -> impl Responder {
    let info = VersionInfo {
        version: env!("VERGEN_GIT_DESCRIBE"),
        commit: env!("VERGEN_GIT_SHA"),
        commit_date: env!("VERGEN_GIT_COMMIT_DATE"),
        build_date: env!("VERGEN_BUILD_DATE"),
        rustc_version: env!("VERGEN_RUSTC_SEMVER"),
        target: env!("VERGEN_CARGO_TARGET_TRIPLE"),
    };
    actix_web::web::Json(info)
}
