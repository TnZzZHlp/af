use serde::Serialize;
use sqlx::{PgPool, QueryBuilder};
use uuid::Uuid;

use super::types::ApiType;

#[derive(Debug, Default)]
pub struct RequestLogFilter {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub request_id: Option<Uuid>,
    pub model: Option<String>,
    pub alias: Option<String>,
    pub provider: Option<String>,
    pub status_code: Option<i32>,
    pub client_ip: Option<String>,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct RequestLog {
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
    pub prompt_tokens: Option<i32>,
    pub completion_tokens: Option<i32>,
    pub total_tokens: Option<i32>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct RequestLogSummary {
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
    // Bodies excluded
    pub request_content_type: Option<String>,
    pub response_content_type: Option<String>,
    pub prompt_tokens: Option<i32>,
    pub completion_tokens: Option<i32>,
    pub total_tokens: Option<i32>,
    #[serde(with = "time::serde::rfc3339")]
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
    pub prompt_tokens: Option<i32>,
    pub completion_tokens: Option<i32>,
    pub total_tokens: Option<i32>,
}

pub async fn fetch_request_logs(
    pool: &PgPool,
    filter: &RequestLogFilter,
) -> anyhow::Result<Vec<RequestLogSummary>> {
    let mut builder = QueryBuilder::new(
        r#"
        SELECT 
            request_id,
            gateway_key_id,
            api_type,
            model,
            alias,
            provider,
            endpoint,
            status_code,
            latency_ms,
            client_ip::text as client_ip,
            user_agent,
            request_content_type,
            response_content_type,
            created_at
        FROM request_logs
        WHERE 1=1
        "#,
    );

    if let Some(request_id) = filter.request_id {
        builder.push(" AND request_id = ");
        builder.push_bind(request_id);
    }

    if let Some(model) = &filter.model {
        builder.push(" AND model ILIKE ");
        builder.push_bind(format!("%{}%", model));
    }

    if let Some(alias) = &filter.alias {
        builder.push(" AND alias ILIKE ");
        builder.push_bind(format!("%{}%", alias));
    }

    if let Some(provider) = &filter.provider {
        builder.push(" AND provider ILIKE ");
        builder.push_bind(format!("%{}%", provider));
    }

    if let Some(status_code) = filter.status_code {
        builder.push(" AND status_code = ");
        builder.push_bind(status_code);
    }

    if let Some(client_ip) = &filter.client_ip {
        builder.push(" AND client_ip::text ILIKE ");
        builder.push_bind(format!("%{}%", client_ip));
    }

    builder.push(" ORDER BY created_at DESC");

    if let Some(limit) = filter.limit {
        builder.push(" LIMIT ");
        builder.push_bind(limit);
    }

    if let Some(offset) = filter.offset {
        builder.push(" OFFSET ");
        builder.push_bind(offset);
    }

    let logs = builder
        .build_query_as::<RequestLogSummary>()
        .fetch_all(pool)
        .await?;

    Ok(logs)
}

pub async fn count_request_logs(
    pool: &PgPool,
    filter: &RequestLogFilter,
) -> anyhow::Result<i64> {
    let mut builder = QueryBuilder::new("SELECT count(*) FROM request_logs WHERE 1=1");

    if let Some(request_id) = filter.request_id {
        builder.push(" AND request_id = ");
        builder.push_bind(request_id);
    }

    if let Some(model) = &filter.model {
        builder.push(" AND model ILIKE ");
        builder.push_bind(format!("%{}%", model));
    }

    if let Some(alias) = &filter.alias {
        builder.push(" AND alias ILIKE ");
        builder.push_bind(format!("%{}%", alias));
    }

    if let Some(provider) = &filter.provider {
        builder.push(" AND provider ILIKE ");
        builder.push_bind(format!("%{}%", provider));
    }

    if let Some(status_code) = filter.status_code {
        builder.push(" AND status_code = ");
        builder.push_bind(status_code);
    }

    if let Some(client_ip) = &filter.client_ip {
        builder.push(" AND client_ip::text ILIKE ");
        builder.push_bind(format!("%{}%", client_ip));
    }

    let count: i64 = builder
        .build_query_scalar::<i64>()
        .fetch_one(pool)
        .await?;

    Ok(count)
}

pub async fn fetch_request_log_detail(
    pool: &PgPool,
    request_id: Uuid,
) -> anyhow::Result<Option<RequestLog>> {
    let log = sqlx::query_as!(
        RequestLog,
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
            prompt_tokens,
            completion_tokens,
            total_tokens,
            created_at
        FROM request_logs
        WHERE request_id = $1
        "#,
        request_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(log)
}

pub async fn record_request(pool: &PgPool, context: &RequestLogContext) -> anyhow::Result<()> {
    let Some(api_type) = context.api_type else {
        return Ok(());
    };

    let client_ip = context
        .client_ip
        .as_deref()
        .and_then(|value| value.parse::<sqlx::types::ipnetwork::IpNetwork>().ok());

    sqlx::query!(
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
            response_content_type,
            prompt_tokens,
            completion_tokens,
            total_tokens
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7,
                $8, $9, $10::inet, $11, $12, $13,
                $14, $15, $16, $17, $18
            )",
        context.request_id,
        context.gateway_key_id,
        api_type as _,
        context.model.as_deref(),
        context.alias.as_deref(),
        context.provider.as_deref(),
        context.endpoint.as_deref(),
        context.status_code,
        context.latency_ms,
        client_ip,
        context.user_agent.as_deref(),
        context.request_body.as_deref(),
        context.response_body.as_deref(),
        context.request_content_type.as_deref(),
        context.response_content_type.as_deref(),
        context.prompt_tokens,
        context.completion_tokens,
        context.total_tokens
    )
    .execute(pool)
    .await?;

    Ok(())
}
