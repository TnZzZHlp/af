use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::types::ApiType;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct RequestLogRow {
    pub request_id: Uuid,
    pub gateway_key_id: Option<Uuid>,
    pub api_type: ApiType,
    pub model: Option<String>,
    pub alias: Option<String>,
    pub provider: Option<String>,
    pub endpoint: Option<String>,
    pub status_code: Option<i32>,
    pub latency_ms: Option<i32>,
    pub client_ip: Option<String>, // Cast from inet
    pub user_agent: Option<String>,
    pub request_body: Option<Vec<u8>>,
    pub response_body: Option<Vec<u8>>,
    pub request_content_type: Option<String>,
    pub response_content_type: Option<String>,
    pub created_at: time::OffsetDateTime,
}

pub struct RequestLogContext {
    pub request_id: Uuid,
    pub gateway_key_id: Option<Uuid>,
    pub api_type: Option<ApiType>,
    pub model: Option<String>,
    pub alias: Option<String>,
    pub provider: Option<String>,
    pub endpoint: Option<String>,
    pub status_code: Option<i32>,
    pub latency_ms: Option<i32>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub request_body: Option<Vec<u8>>,
    pub response_body: Option<Vec<u8>>,
    pub request_content_type: Option<String>,
    pub response_content_type: Option<String>,
}

pub struct RequestLogInsert {
    pub request_id: Uuid,
    pub gateway_key_id: Option<Uuid>,
    pub api_type: ApiType,
    pub model: Option<String>,
    pub alias: Option<String>,
    pub provider: Option<String>,
    pub endpoint: Option<String>,
    pub status_code: Option<i32>,
    pub latency_ms: Option<i32>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub request_body: Option<Vec<u8>>,
    pub response_body: Option<Vec<u8>>,
    pub request_content_type: Option<String>,
    pub response_content_type: Option<String>,
}

pub async fn fetch_request_logs(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> anyhow::Result<Vec<RequestLogRow>> {
    let logs = sqlx::query_as!(
        RequestLogRow,
        r#"
        SELECT 
            request_id,
            gateway_key_id,
            api_type as "api_type: ApiType",
            model,
            alias,
            provider,
            endpoint,
            status_code,
            latency_ms,
            client_ip::text as client_ip,
            user_agent,
            request_body,
            response_body,
            request_content_type,
            response_content_type,
            created_at
        FROM request_logs
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    Ok(logs)
}

pub async fn record_request(pool: &PgPool, context: &RequestLogContext) -> anyhow::Result<()> {
    let Some(api_type) = context.api_type else {
        return Ok(());
    };

    sqlx::query(
        "INSERT INTO request_logs (
            request_id,
            gateway_key_id,
            api_type,
            model,
            alias,
            provider,
            endpoint,
            status_code,
            latency_ms,
            client_ip,
            user_agent,
            request_body,
            response_body,
            request_content_type,
            response_content_type
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7,
                $8, $9, $10::inet, $11, $12, $13,
                $14, $15
            )",
    )
    .bind(context.request_id)
    .bind(context.gateway_key_id)
    .bind(api_type)
    .bind(context.model.as_deref())
    .bind(context.alias.as_deref())
    .bind(context.provider.as_deref())
    .bind(context.endpoint.as_deref())
    .bind(context.status_code)
    .bind(context.latency_ms)
    .bind(context.client_ip.as_deref())
    .bind(context.user_agent.as_deref())
    .bind(context.request_body.as_deref())
    .bind(context.response_body.as_deref())
    .bind(context.request_content_type.as_deref())
    .bind(context.response_content_type.as_deref())
    .execute(pool)
    .await?;

    Ok(())
}
