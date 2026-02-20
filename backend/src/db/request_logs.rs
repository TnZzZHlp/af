use serde::Serialize;
use sqlx::{PgPool, QueryBuilder};
use uuid::Uuid;

use super::types::ApiType;
use crate::utils::request_body_hash::hash_request_body_hex;

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
    pub cache_layer: Option<String>,
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
    pub cache_layer: Option<String>,
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

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct CachedResponse {
    pub source_request_log_id: i64,
    pub status_code: i32,
    pub response_body: Vec<u8>,
    pub response_content_type: Option<String>,
}

fn append_filters<'a>(
    builder: &mut QueryBuilder<'a, sqlx::Postgres>,
    filter: &'a RequestLogFilter,
    rl_alias: &str,
    main_alias: &str,
) {
    if let Some(model) = &filter.model {
        builder.push(format!(" AND {}.model ILIKE ", rl_alias));
        builder.push_bind(format!("%{}%", model));
    }

    if let Some(alias) = &filter.alias {
        builder.push(format!(" AND {}.alias ILIKE ", rl_alias));
        builder.push_bind(format!("%{}%", alias));
    }

    if let Some(provider) = &filter.provider {
        builder.push(format!(" AND {}.provider ILIKE ", rl_alias));
        builder.push_bind(format!("%{}%", provider));
    }

    if let Some(status_code) = filter.status_code {
        builder.push(format!(" AND {}.status_code = ", rl_alias));
        builder.push_bind(status_code);
    }

    if let Some(client_ip) = &filter.client_ip {
        builder.push(format!(" AND {}.client_ip::text ILIKE ", main_alias));
        builder.push_bind(format!("%{}%", client_ip));
    }

    if let Some(request_id) = filter.request_id {
        builder.push(format!(" AND {}.request_id = ", main_alias));
        builder.push_bind(request_id);
    }
}

pub async fn fetch_request_logs(
    pool: &PgPool,
    filter: &RequestLogFilter,
) -> anyhow::Result<Vec<RequestLogSummary>> {
    let mut builder = QueryBuilder::new("SELECT * FROM (");

    // Part 1: request_logs
    builder.push(
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
            prompt_tokens,
            completion_tokens,
            total_tokens,
            created_at,
            NULL::text as cache_layer
        FROM request_logs
        WHERE 1=1
        "#,
    );
    append_filters(&mut builder, filter, "request_logs", "request_logs");

    builder.push(" UNION ALL ");

    // Part 2: cache_log
    builder.push(
        r#"
        SELECT
            cl.request_id,
            cl.gateway_key_id,
            rl.api_type,
            rl.model,
            rl.alias,
            rl.provider,
            rl.endpoint,
            rl.status_code,
            cl.latency_ms,
            cl.client_ip::text as client_ip,
            cl.user_agent,
            rl.request_content_type,
            rl.response_content_type,
            rl.prompt_tokens,
            rl.completion_tokens,
            rl.total_tokens,
            cl.created_at,
            cl.cache_layer
        FROM cache_log cl
        JOIN request_logs rl ON cl.source_request_log_id = rl.id
        WHERE 1=1
        "#,
    );
    append_filters(&mut builder, filter, "rl", "cl");

    builder.push(") as combined ORDER BY created_at DESC");

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

pub async fn count_request_logs(pool: &PgPool, filter: &RequestLogFilter) -> anyhow::Result<i64> {
    let mut builder = QueryBuilder::new("SELECT count(*) FROM (");

    // Part 1: request_logs
    builder.push(
        r#"
        SELECT request_id
        FROM request_logs
        WHERE 1=1
        "#,
    );
    append_filters(&mut builder, filter, "request_logs", "request_logs");

    builder.push(" UNION ALL ");

    // Part 2: cache_log
    builder.push(
        r#"
        SELECT cl.request_id
        FROM cache_log cl
        JOIN request_logs rl ON cl.source_request_log_id = rl.id
        WHERE 1=1
        "#,
    );
    append_filters(&mut builder, filter, "rl", "cl");

    builder.push(") as combined");

    let count: i64 = builder.build_query_scalar::<i64>().fetch_one(pool).await?;

    Ok(count)
}

pub async fn fetch_request_log_detail(
    pool: &PgPool,
    request_id: Uuid,
) -> anyhow::Result<Option<RequestLog>> {
    let log = sqlx::query_as::<_, RequestLog>(
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
            request_body,
            response_body,
            request_content_type,
            response_content_type,
            prompt_tokens,
            completion_tokens,
            total_tokens,
            created_at,
            NULL::text as cache_layer
        FROM request_logs
        WHERE request_id = $1
        UNION ALL
        SELECT
            cl.request_id,
            cl.gateway_key_id,
            rl.api_type,
            rl.model,
            rl.alias,
            rl.provider,
            rl.endpoint,
            rl.status_code,
            cl.latency_ms,
            cl.client_ip::text as client_ip,
            cl.user_agent,
            rl.request_body,
            rl.response_body,
            rl.request_content_type,
            rl.response_content_type,
            rl.prompt_tokens,
            rl.completion_tokens,
            rl.total_tokens,
            cl.created_at,
            cl.cache_layer
        FROM cache_log cl
        JOIN request_logs rl ON cl.source_request_log_id = rl.id
        WHERE cl.request_id = $1
        "#,
    )
    .bind(request_id)
    .fetch_optional(pool)
    .await?;

    Ok(log)
}

pub async fn record_request(pool: &PgPool, context: &RequestLogContext) -> anyhow::Result<()> {
    let Some(api_type) = context.api_type else {
        return Ok(());
    };
    let request_body_hash = context.request_body.as_deref().map(hash_request_body_hex);

    let client_ip = context
        .client_ip
        .as_deref()
        .and_then(|value| value.parse::<sqlx::types::ipnetwork::IpNetwork>().ok());

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
            request_body_hash,
            response_body,
            request_content_type,
            response_content_type,
            prompt_tokens,
            completion_tokens,
            total_tokens
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7,
            $8, $9, $10::inet, $11, $12, $13,
            $14, $15, $16, $17, $18, $19
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
    .bind(client_ip)
    .bind(context.user_agent.as_deref())
    .bind(context.request_body.as_deref())
    .bind(request_body_hash.as_deref())
    .bind(context.response_body.as_deref())
    .bind(context.request_content_type.as_deref())
    .bind(context.response_content_type.as_deref())
    .bind(context.prompt_tokens)
    .bind(context.completion_tokens)
    .bind(context.total_tokens)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn find_cached_response(
    pool: &PgPool,
    request_body_hash: &str,
) -> anyhow::Result<Option<CachedResponse>> {
    let cached = sqlx::query_as::<_, CachedResponse>(
        r#"
        SELECT
            id as source_request_log_id,
            status_code,
            response_body,
            response_content_type
        FROM request_logs
        WHERE request_body_hash = $1
          AND status_code BETWEEN 200 AND 299
          AND response_body IS NOT NULL
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(request_body_hash)
    .fetch_optional(pool)
    .await?;

    Ok(cached)
}
