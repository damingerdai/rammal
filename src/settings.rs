use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct ProxySettings {
    pub upstream: String,
    pub cache_dir: String,
    pub connect_timeout: u64,
    pub request_timeout: u64,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub proxy: ProxySettings,
}

impl Settings {
    pub fn load() -> Result<Self, ConfigError> {
        Config::builder()
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 8000)?
            .set_default("proxy.upstream", "https://registry-1.docker.io")?
            .set_default("proxy.cache_dir", "./docker_cache")?
            .set_default("proxy.connect_timeout", 10)?
            .set_default("proxy.request_timeout", 30)?
            .add_source(File::with_name("config/default").required(false))
            .add_source(
                Environment::with_prefix("RAMMAL")
                    .prefix_separator("_")
                    .separator("_"),
            )
            .build()?
            .try_deserialize()
    }
}
