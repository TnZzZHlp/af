mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod router;
mod services;
mod state;

use std::net::SocketAddr;

use clap::Parser;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

use crate::{
    config::load_config,
    services::{
        auth::LoginProtection, openai::OpenAiService, rate_limit::RateLimiter,
        response_cache::ResponseCache,
    },
    state::AppState,
};

#[derive(Debug, Parser)]
#[command(name = "backend", version, about = "AI gateway backend")]
struct Cli {
    /// Log level for backend module (e.g. trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,
    /// Full tracing filter (overrides log_level)
    #[arg(long)]
    log_filter: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let fallback_filter = cli
        .log_filter
        .unwrap_or_else(|| format!("backend={},info", cli.log_level));
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(fallback_filter));

    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let config = load_config()?;
    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await?;

    tracing::info!("running database migrations");
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("database migrations complete");

    let http_client = reqwest::Client::new();
    let openai = OpenAiService::new(pool.clone(), http_client);
    let rate_limiter = RateLimiter::new();
    let login_protection = LoginProtection::new();
    let response_cache = ResponseCache::new();
    let state = AppState {
        pool,
        openai,
        jwt_secret: config.jwt_secret,
        rate_limiter,
        login_protection,
        response_cache,
    };
    let app = router::app(state);

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .expect("valid listen address");
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
