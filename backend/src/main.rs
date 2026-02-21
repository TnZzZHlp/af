mod config;
mod constants;
mod db;
mod error;
mod handlers;
mod middleware;
mod router;
mod services;
mod state;
mod utils;

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use anyhow::anyhow;
use clap::Parser;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

use crate::{
    config::load_config,
    services::{
        auth::LoginProtection, background::BackgroundTasks, openai::OpenAiService,
        rate_limit::RateLimiter, response_cache::ResponseCache,
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

#[derive(Clone, Copy, Debug)]
enum ShutdownSignal {
    Sigint,
    Sigterm,
}

impl ShutdownSignal {
    fn as_str(self) -> &'static str {
        match self {
            Self::Sigint => "SIGINT",
            Self::Sigterm => "SIGTERM",
        }
    }
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

    let background_tasks = BackgroundTasks::new();
    let http_client = reqwest::Client::new();
    let openai = OpenAiService::new(pool.clone(), http_client, background_tasks.clone());
    let rate_limiter = RateLimiter::new();
    let login_protection = LoginProtection::new();
    let response_cache = ResponseCache::new();
    let state = AppState {
        pool: pool.clone(),
        openai,
        jwt_secret: config.jwt_secret,
        rate_limiter,
        login_protection,
        response_cache,
        background_tasks: background_tasks.clone(),
    };
    let app = router::app(state);

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .expect("valid listen address");
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let server_shutdown_token = background_tasks.token();
    let server = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(server_shutdown_token.clone().cancelled_owned())
    .into_future();
    tokio::pin!(server);

    let shutdown_timeout = Duration::from_secs(config.server.graceful_shutdown_timeout_secs);
    let mut shutdown_started_at = None;
    let mut server_result: anyhow::Result<()> = Ok(());

    tokio::select! {
        result = &mut server => {
            server_result = result.map_err(anyhow::Error::from);
        }
        signal = wait_for_shutdown_signal() => {
            let signal = signal?;
            shutdown_started_at = Some(Instant::now());

            tracing::info!(
                signal = signal.as_str(),
                timeout_secs = config.server.graceful_shutdown_timeout_secs,
                "shutdown signal received, stopping new connections"
            );
            server_shutdown_token.cancel();

            tracing::info!(
                timeout_secs = config.server.graceful_shutdown_timeout_secs,
                "waiting for in-flight requests to complete"
            );
            match tokio::time::timeout(shutdown_timeout, &mut server).await {
                Ok(result) => {
                    server_result = result.map_err(anyhow::Error::from);
                    tracing::info!("http server stopped accepting connections");
                }
                Err(_) => {
                    tracing::warn!(
                        timeout_secs = config.server.graceful_shutdown_timeout_secs,
                        "graceful shutdown timed out while waiting for in-flight requests"
                    );
                }
            }
        }
    }

    background_tasks.begin_shutdown();
    let cleanup_timeout = shutdown_started_at
        .map(|started_at| {
            shutdown_timeout.saturating_sub(Instant::now().saturating_duration_since(started_at))
        })
        .unwrap_or(shutdown_timeout);

    if cleanup_timeout.is_zero() {
        let pending = background_tasks.pending_count();
        if pending > 0 {
            tracing::warn!(
                pending_tasks = pending,
                "shutdown timeout reached; background tasks will be forcefully terminated"
            );
        }
    } else {
        tracing::info!(
            timeout_ms = cleanup_timeout.as_millis(),
            "waiting for background tasks to finish"
        );
        if !background_tasks.wait(cleanup_timeout).await {
            let pending = background_tasks.pending_count();
            tracing::warn!(
                pending_tasks = pending,
                "background tasks did not finish before timeout and will be forcefully terminated"
            );
        }
    }

    let db_close_timeout = shutdown_started_at
        .map(|started_at| {
            shutdown_timeout.saturating_sub(Instant::now().saturating_duration_since(started_at))
        })
        .unwrap_or(shutdown_timeout);
    if db_close_timeout.is_zero() {
        tracing::warn!("shutdown timeout reached before database pool could be closed cleanly");
    } else {
        tracing::info!(
            timeout_ms = db_close_timeout.as_millis(),
            "closing database pool"
        );
        if tokio::time::timeout(db_close_timeout, pool.close())
            .await
            .is_err()
        {
            tracing::warn!("database pool close timed out");
        }
    }

    tracing::info!("shutdown complete");
    server_result?;

    Ok(())
}

#[cfg(unix)]
async fn wait_for_shutdown_signal() -> anyhow::Result<ShutdownSignal> {
    use tokio::signal::unix::{SignalKind, signal};

    let mut sigterm = signal(SignalKind::terminate())
        .map_err(|err| anyhow!("failed to install SIGTERM handler: {err}"))?;

    tokio::select! {
        result = tokio::signal::ctrl_c() => {
            result.map_err(|err| anyhow!("failed to install SIGINT handler: {err}"))?;
            Ok(ShutdownSignal::Sigint)
        }
        _ = sigterm.recv() => Ok(ShutdownSignal::Sigterm),
    }
}

#[cfg(not(unix))]
async fn wait_for_shutdown_signal() -> anyhow::Result<ShutdownSignal> {
    tokio::signal::ctrl_c()
        .await
        .map_err(|err| anyhow!("failed to install SIGINT handler: {err}"))?;
    Ok(ShutdownSignal::Sigint)
}
