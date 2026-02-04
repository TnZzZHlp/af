use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

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

    sqlx::query(
        "INSERT INTO request_logs (\n            request_id,\n            gateway_key_id,\n            api_type,\n            model,\n            alias,\n            provider,\n            endpoint,\n            status_code,\n            latency_ms,\n            error_code,\n            error_message,\n            client_ip,\n            user_agent,\n            request_headers,\n            request_body,\n            response_headers,\n            response_body\n         ) VALUES (\n            $1, $2, $3, $4, $5, $6, $7,\n            $8, $9, $10, $11, $12, $13, $14, $15, $16, $17\n         )",
    )
    .bind(context.request_id)
    .bind(context.gateway_key_id)
    .bind(api_type.as_str())
    .bind(context.model.as_deref())
    .bind(context.alias.as_deref())
    .bind(context.provider.as_deref())
    .bind(context.endpoint.as_deref())
    .bind(context.status_code)
    .bind(context.latency_ms)
    .bind(context.error_code.as_deref())
    .bind(context.error_message.as_deref())
    .bind(context.client_ip.as_deref())
    .bind(context.user_agent.as_deref())
    .bind(context.request_headers.as_ref())
    .bind(context.request_body.as_deref())
    .bind(context.response_headers.as_ref())
    .bind(context.response_body.as_deref())
    .execute(pool)
    .await?;

    Ok(())
}
