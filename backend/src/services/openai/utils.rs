use std::time::Instant;

use serde_json::Value;
use uuid::Uuid;

use crate::{
    db::{request_logs::RequestLogContext, types::ApiType},
    middleware::auth::GatewayKeyId,
};

#[derive(Clone)]
pub(super) struct RequestContext {
    pub request_id: Uuid,
    pub gateway_key_id: GatewayKeyId,
    pub api_type: ApiType,
    pub model: String,
    pub alias: String,
    pub provider: String,
    pub endpoint: String,
    pub start: Instant,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub request_body: Vec<u8>,
}

impl RequestContext {
    pub(super) fn build_log_context(
        &self,
        status_code: Option<i32>,
        response_body: Option<Vec<u8>>,
        response_content_type: Option<String>,
        prompt_tokens: Option<i32>,
        completion_tokens: Option<i32>,
        total_tokens: Option<i32>,
    ) -> RequestLogContext {
        RequestLogContext {
            request_id: self.request_id,
            gateway_key_id: Some(self.gateway_key_id.0),
            api_type: Some(self.api_type),
            model: Some(self.model.clone()),
            alias: Some(self.alias.clone()),
            provider: Some(self.provider.clone()),
            endpoint: Some(self.endpoint.clone()),
            status_code,
            latency_ms: Some(elapsed_ms(self.start)),
            client_ip: self.client_ip.clone(),
            user_agent: self.user_agent.clone(),
            request_body: Some(self.request_body.clone()),
            response_body,
            request_content_type: Some("application/json".to_string()),
            response_content_type,
            prompt_tokens,
            completion_tokens,
            total_tokens,
        }
    }
}

pub(super) fn elapsed_ms(start: Instant) -> i32 {
    i32::try_from(start.elapsed().as_millis()).unwrap_or(i32::MAX)
}

pub(super) fn extract_usage(
    body: &[u8],
    api_type: ApiType,
) -> (Option<i32>, Option<i32>, Option<i32>) {
    // Try to parse as JSON first (non-streaming)
    if let Ok(json) = serde_json::from_slice::<Value>(body) {
        match api_type {
            ApiType::OpenAiChatCompletions
            | ApiType::OpenAiEmbeddings
            | ApiType::OpenAiResponses => {
                if let Some(usage) = json.get("usage") {
                    let prompt = usage
                        .get("prompt_tokens")
                        .and_then(Value::as_i64)
                        .map(|v| v as i32);
                    let completion = usage
                        .get("completion_tokens")
                        .and_then(Value::as_i64)
                        .map(|v| v as i32);
                    let total = usage
                        .get("total_tokens")
                        .and_then(Value::as_i64)
                        .map(|v| v as i32);
                    return (prompt, completion, total);
                }
            }
            ApiType::AnthropicMessages => {
                if let Some(usage) = json.get("usage") {
                    let input = usage
                        .get("input_tokens")
                        .and_then(Value::as_i64)
                        .map(|v| v as i32);
                    let output = usage
                        .get("output_tokens")
                        .and_then(Value::as_i64)
                        .map(|v| v as i32);
                    let total = if let (Some(i), Some(o)) = (input, output) {
                        Some(i + o)
                    } else {
                        None
                    };
                    return (input, output, total);
                }
            }
            ApiType::OpenAiModels => {}
        }
    }

    // parsing as JSON failed, maybe it is a streaming response (SSE)
    // We look for the "usage" field in the last few lines
    // This is a rough heuristic.
    let body_str = String::from_utf8_lossy(body);
    // OpenAi usage in stream: data: {"...": ..., "usage": {...}}
    // It might be one of the last data chunks.
    for line in body_str.lines().rev() {
        if let Some(json_str) = line.strip_prefix("data: ") {
            if json_str == "[DONE]" {
                continue;
            }
            if let Ok(json) = serde_json::from_str::<Value>(json_str)
                && let Some(usage) = json.get("usage")
            {
                // Same extraction logic as above
                match api_type {
                    ApiType::OpenAiChatCompletions
                    | ApiType::OpenAiEmbeddings
                    | ApiType::OpenAiResponses => {
                        let prompt = usage
                            .get("prompt_tokens")
                            .and_then(Value::as_i64)
                            .map(|v| v as i32);
                        let completion = usage
                            .get("completion_tokens")
                            .and_then(Value::as_i64)
                            .map(|v| v as i32);
                        let total = usage
                            .get("total_tokens")
                            .and_then(Value::as_i64)
                            .map(|v| v as i32);
                        if prompt.is_some() || completion.is_some() || total.is_some() {
                            return (prompt, completion, total);
                        }
                    }
                    ApiType::AnthropicMessages => {
                        // Anthropic streaming usage is different (message_delta), but for now let's see if usage is present
                        if let Some(input) = usage
                            .get("input_tokens")
                            .and_then(Value::as_i64)
                            .map(|v| v as i32)
                        {
                            let output = usage
                                .get("output_tokens")
                                .and_then(Value::as_i64)
                                .map(|v| v as i32);
                            let total = output.map(|o| input + o);
                            return (Some(input), output, total);
                        }
                    }
                    ApiType::OpenAiModels => {}
                }
            }
        }
    }

    (None, None, None)
}
