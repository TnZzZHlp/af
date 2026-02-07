use sqlx::PgPool;
use uuid::Uuid;

use crate::db::request_logs;
pub use crate::db::request_logs::RequestLogRow;
pub use crate::db::request_logs::RequestLogSummary;
pub use crate::db::types::ApiType;

pub async fn fetch_request_logs(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> anyhow::Result<Vec<RequestLogSummary>> {
    request_logs::fetch_request_logs(pool, limit, offset).await
}

pub async fn fetch_request_log_detail(
    pool: &PgPool,
    request_id: Uuid,
) -> anyhow::Result<Option<RequestLogRow>> {
    request_logs::fetch_request_log_detail(pool, request_id).await
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
    };

    request_logs::record_request(pool, &db_context).await
}
