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
            .add_source(File::with_name("config/default"))
            .add_source(
                Environment::with_prefix("RAMMAL")
                    .prefix_separator("_")
                    .separator("_"),
            )
            .build()?
            .try_deserialize()
    }
}
