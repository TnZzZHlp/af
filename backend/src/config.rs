use std::env;

use anyhow::anyhow;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt_secret: String,
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
    let server_host = env::var("SERVER_HOST").unwrap_or("0.0.0.0".to_string());
    let server_port = env::var("SERVER_PORT")
        .unwrap_or("30002".to_string())
        .parse::<u16>()
        .map_err(|err| anyhow!("SERVER_PORT must be a u16: {err}"))?;

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres@192.168.255.201:5432/af".to_string());
    let database_max_connections = env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .map(|value| {
            value
                .parse::<u32>()
                .map_err(|err| anyhow!("DATABASE_MAX_CONNECTIONS must be a u32: {err}"))
        })
        .transpose()?
        .unwrap_or(10);

    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or("F0oA/t+6Ia2rs/oWEvCjOUYk67kWKhOISNDzrDP6WHM=".to_string());

    Ok(AppConfig {
        server: ServerConfig {
            host: server_host,
            port: server_port,
        },
        database: DatabaseConfig {
            url: database_url,
            max_connections: database_max_connections,
        },
        jwt_secret,
    })
}
