use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::request_logs;

#[derive(Debug, Clone, Copy)]
pub enum ApiType {
    OpenAiChatCompletions,
    OpenAiResponses,
    AnthropicMessages,
}

impl ApiType {
    pub fn as_str(self) -> &'static str {
        match self {
            ApiType::OpenAiChatCompletions => "openai_chat_completions",
            ApiType::OpenAiResponses => "openai_responses",
            ApiType::AnthropicMessages => "anthropic_messages",
        }
    }
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
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub request_headers: Option<Value>,
    pub request_body: Option<Vec<u8>>,
    pub response_headers: Option<Value>,
    pub response_body: Option<Vec<u8>>,
}

pub struct RequestLogInsert {
    pub request_id: Uuid,
    pub gateway_key_id: Option<Uuid>,
    pub api_type: String,
    pub model: Option<String>,
    pub alias: Option<String>,
    pub provider: Option<String>,
    pub endpoint: Option<String>,
    pub status_code: Option<i32>,
    pub latency_ms: Option<i32>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub request_headers: Option<Value>,
    pub request_body: Option<Vec<u8>>,
    pub response_headers: Option<Value>,
    pub response_body: Option<Vec<u8>>,
}

pub async fn record_request(pool: &PgPool, context: &RequestLogContext) -> anyhow::Result<()> {
    let Some(api_type) = context.api_type else {
        return Ok(());
    };

    let db_context = request_logs::RequestLogContext {
        request_id: context.request_id,
        gateway_key_id: context.gateway_key_id,
        api_type: Some(match api_type {
            ApiType::OpenAiChatCompletions => request_logs::ApiType::OpenAiChatCompletions,
            ApiType::OpenAiResponses => request_logs::ApiType::OpenAiResponses,
            ApiType::AnthropicMessages => request_logs::ApiType::AnthropicMessages,
        }),
        model: context.model.clone(),
        alias: context.alias.clone(),
        provider: context.provider.clone(),
        endpoint: context.endpoint.clone(),
        status_code: context.status_code,
        latency_ms: context.latency_ms,
        error_code: context.error_code.clone(),
        error_message: context.error_message.clone(),
        client_ip: context.client_ip.clone(),
        user_agent: context.user_agent.clone(),
        request_headers: context.request_headers.clone(),
        request_body: context.request_body.clone(),
        response_headers: context.response_headers.clone(),
        response_body: context.response_body.clone(),
    };

    request_logs::record_request(pool, &db_context).await
}
