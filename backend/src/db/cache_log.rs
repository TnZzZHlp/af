use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;

pub struct CacheLogContext {
    pub request_id: Uuid,
    pub source_request_log_id: Option<i64>,
    pub gateway_key_id: Option<Uuid>,
    pub cache_layer: &'static str,
    pub latency_ms: Option<i32>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
}

pub async fn record_cache_event(pool: &PgPool, context: &CacheLogContext) -> AppResult<()> {
    let client_ip = context
        .client_ip
        .as_deref()
        .and_then(|value| value.parse::<sqlx::types::ipnetwork::IpNetwork>().ok());

    sqlx::query(
        "INSERT INTO cache_log (
            request_id,
            source_request_log_id,
            gateway_key_id,
            cache_layer,
            latency_ms,
            client_ip,
            user_agent
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7
        )",
    )
    .bind(context.request_id)
    .bind(context.source_request_log_id)
    .bind(context.gateway_key_id)
    .bind(context.cache_layer)
    .bind(context.latency_ms)
    .bind(client_ip)
    .bind(context.user_agent.as_deref())
    .execute(pool)
    .await?;

    Ok(())
}
