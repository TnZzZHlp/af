use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(type_name = "api_type")]
pub enum ApiType {
    #[sqlx(rename = "openai_chat_completions")]
    OpenAiChatCompletions,
    #[sqlx(rename = "openai_responses")]
    OpenAiResponses,
    #[sqlx(rename = "anthropic_messages")]
    AnthropicMessages,
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
}

pub async fn record_request(pool: &PgPool, context: &RequestLogContext) -> anyhow::Result<()> {
    let Some(api_type) = context.api_type else {
        return Ok(());
    };

    sqlx::query(
        "INSERT INTO request_logs (\n            request_id,\n            gateway_key_id,\n            api_type,\n            model,\n            alias,\n            provider,\n            endpoint,\n            status_code,\n            latency_ms,\n            client_ip,\n            user_agent,\n            request_body,\n            response_body\n         ) VALUES (\n            $1, $2, $3, $4, $5, $6, $7,\n            $8, $9, $10, $11, $12, $13\n         )",
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
    .execute(pool)
    .await?;

    Ok(())
}
