mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod router;
mod services;
mod state;

use std::net::SocketAddr;

use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

use crate::{config::load_config, services::openai::OpenAiService, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = load_config()?;
    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let http_client = reqwest::Client::new();
    let openai = OpenAiService::new(pool.clone(), http_client);
    let state = AppState {
        pool,
        openai,
        jwt_secret: config.jwt_secret,
    };
    let app = router::app(state);

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .expect("valid listen address");
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
