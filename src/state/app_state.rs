use awc::Client;

pub(crate) struct AppState {
    pub(crate) client: Client,
    pub(crate) upstream: String,
    pub(crate) cache_dir: String,
}
