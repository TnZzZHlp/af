use sqlx::PgPool;
use uuid::Uuid;

use super::types::ApiType;

pub struct CacheLogContext {
    pub request_id: Uuid,
    pub source_request_log_id: Option<i64>,
    pub gateway_key_id: Option<Uuid>,
    pub api_type: ApiType,
    pub cache_layer: &'static str,
    pub hit: bool,
    pub request_body_hash: Option<String>,
    pub request_body_size: Option<i32>,
    pub response_body_size: Option<i32>,
    pub latency_ms: Option<i32>,
}

pub async fn record_cache_event(pool: &PgPool, context: &CacheLogContext) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO cache_log (
            request_id,
            source_request_log_id,
            gateway_key_id,
            api_type,
            cache_layer,
            hit,
            request_body_hash,
            request_body_size,
            response_body_size,
            latency_ms
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10
        )",
    )
    .bind(context.request_id)
    .bind(context.source_request_log_id)
    .bind(context.gateway_key_id)
    .bind(context.api_type)
    .bind(context.cache_layer)
    .bind(context.hit)
    .bind(context.request_body_hash.as_deref())
    .bind(context.request_body_size)
    .bind(context.response_body_size)
    .bind(context.latency_ms)
    .execute(pool)
    .await?;

    Ok(())
}
