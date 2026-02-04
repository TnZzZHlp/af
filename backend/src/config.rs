use serde::Deserialize;

use crate::config;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

pub fn load_config() -> anyhow::Result<AppConfig> {
    let settings = config::Config::builder()
        .set_default(
            "database.url",
            "postgres://postgres@192.168.255.201:5432/af",
        )?
        .set_default("database.max_connections", 10u32)?
        .build()?;
    Ok(settings.try_deserialize()?)
}
