use sqlx::PgPool;
use uuid::Uuid;

use crate::db::request_logs::{
    self, RequestLog, RequestLogContext, RequestLogFilter, RequestLogSummary,
};

pub async fn fetch_request_logs(
    pool: &PgPool,
    filter: &RequestLogFilter,
) -> anyhow::Result<Vec<RequestLogSummary>> {
    request_logs::fetch_request_logs(pool, filter).await
}

pub async fn count_request_logs(pool: &PgPool, filter: &RequestLogFilter) -> anyhow::Result<i64> {
    request_logs::count_request_logs(pool, filter).await
}

pub async fn fetch_request_log_detail(
    pool: &PgPool,
    request_id: Uuid,
) -> anyhow::Result<Option<RequestLog>> {
    request_logs::fetch_request_log_detail(pool, request_id).await
}

pub async fn record_request(pool: &PgPool, context: &RequestLogContext) -> anyhow::Result<()> {
    let Some(api_type) = context.api_type else {
        return Ok(());
    };

    let db_context = request_logs::RequestLogContext {
        request_id: context.request_id,
        gateway_key_id: context.gateway_key_id,
        api_type: Some(api_type),
        model: context.model.clone(),
        alias: context.alias.clone(),
        provider: context.provider.clone(),
        endpoint: context.endpoint.clone(),
        status_code: context.status_code,
        latency_ms: context.latency_ms,
        client_ip: context.client_ip.clone(),
        user_agent: context.user_agent.clone(),
        request_body: context.request_body.clone(),
        response_body: context.response_body.clone(),
        request_content_type: context.request_content_type.clone(),
        response_content_type: context.response_content_type.clone(),
        prompt_tokens: context.prompt_tokens,
        completion_tokens: context.completion_tokens,
        total_tokens: context.total_tokens,
    };

    request_logs::record_request(pool, &db_context).await
}
