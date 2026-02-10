use std::time::Instant;

use axum::{
    body::Body,
    http::{Response, header},
};
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    db::{
        provider_keys::{self, UpdateKeyParams},
        request_logs::RequestLogContext,
        types::ApiType,
    },
    error::{AppError, AppResult},
    middleware::{auth::GatewayKeyId, request_log::ClientInfo},
    services::{logging, providers, routing},
};

#[derive(Debug, Deserialize)]
struct ProviderModel {
    id: String,
    object: String,
    created: u64,
    owned_by: String,
}

#[derive(Debug, Serialize)]
pub struct Model {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
    pub api_type: ApiType,
}

#[derive(Clone)]
pub struct OpenAiService {
    pool: PgPool,
    http_client: Client,
}

impl OpenAiService {
    pub fn new(pool: PgPool, http_client: Client) -> Self {
        Self { pool, http_client }
    }

    pub async fn list_models(&self, provider_id: Uuid) -> AppResult<Vec<Model>> {
        let endpoints = providers::list_endpoints_by_provider(&self.pool, provider_id)
            .await
            .map_err(AppError::Internal)?;

        let endpoint = endpoints
            .iter()
            .find(|e| matches!(e.api_type, ApiType::OpenAiModels))
            .ok_or_else(|| {
                AppError::BadRequest(
                    "No compatible endpoint found (OpenAiModels required)".to_string(),
                )
            })?;

        let url = endpoint.url.clone();
        let api_type = endpoint.api_type;

        let keys = providers::list_keys_by_provider(&self.pool, provider_id)
            .await
            .map_err(AppError::Internal)?;
        let key = keys
            .iter()
            .find(|k| k.enabled)
            .ok_or_else(|| AppError::BadRequest("No enabled key found".to_string()))?;

        tracing::debug!(%url, "fetching models from provider");

        let response = self
            .http_client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", key.key))
            .send()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            tracing::error!(%status, %text, "provider list models failed");
            return Err(AppError::BadRequest(format!(
                "Provider returned error: {}",
                status
            )));
        }

        let body: Value = response
            .json()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        let data = body
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                AppError::Internal(
                    anyhow::anyhow!(
                        "Invalid response format: 'data' field missing or not an array"
                    ),
                )
            })?;

        let provider_models: Vec<ProviderModel> = serde_json::from_value(Value::Array(data.clone()))
            .map_err(|e| AppError::Internal(e.into()))?;

        let models = provider_models
            .into_iter()
            .map(|pm| Model {
                id: pm.id,
                object: pm.object,
                created: pm.created,
                owned_by: pm.owned_by,
                api_type,
            })
            .collect();

        Ok(models)
    }

    pub async fn chat_completions(
        &self,
        gateway_key_id: GatewayKeyId,
        payload: Value,
        client_info: ClientInfo,
    ) -> AppResult<Response<Body>> {
        self.process_request(
            gateway_key_id,
            payload,
            client_info,
            ApiType::OpenAiChatCompletions,
        )
        .await
    }

    pub async fn responses(
        &self,
        gateway_key_id: GatewayKeyId,
        payload: Value,
        client_info: ClientInfo,
    ) -> AppResult<Response<Body>> {
        self.process_request(
            gateway_key_id,
            payload,
            client_info,
            ApiType::OpenAiResponses,
        )
        .await
    }

    pub async fn anthropic_messages(
        &self,
        gateway_key_id: GatewayKeyId,
        payload: Value,
        client_info: ClientInfo,
    ) -> AppResult<Response<Body>> {
        self.process_request(
            gateway_key_id,
            payload,
            client_info,
            ApiType::AnthropicMessages,
        )
        .await
    }

    async fn process_request(
        &self,
        gateway_key_id: GatewayKeyId,
        mut payload: Value,
        client_info: ClientInfo,
        api_type: ApiType,
    ) -> AppResult<Response<Body>> {
        let request_id = Uuid::now_v7();
        let start = Instant::now();

        let ClientInfo {
            client_ip,
            user_agent,
        } = client_info;

        tracing::debug!(%request_id, %api_type, "received request");

        let payload_object = payload
            .as_object_mut()
            .ok_or_else(|| AppError::BadRequest("payload must be a JSON object".to_string()))?;
        let model = payload_object
            .get("model")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| AppError::BadRequest("model is required".to_string()))?;

        tracing::debug!(%model, "extracted model from payload");

        let route = match routing::resolve_route(
            &self.pool,
            gateway_key_id.0,
            &model,
            api_type,
        )
        .await
        {
            Ok(route) => route,
            Err(err) => match err.downcast::<AppError>() {
                Ok(app_err) => return Err(app_err),
                Err(other_err) => return Err(AppError::Internal(other_err)),
            },
        };

        let pool = self.pool.clone();
        let provider_id = route.provider_id;
        tokio::spawn(async move {
            let _ = providers::increment_usage_count(&pool, provider_id).await;
        });

        let pool = self.pool.clone();
        let key_id = route.provider_key.id;
        tokio::spawn(async move {
            let _ = provider_keys::increment_usage_count(&pool, key_id).await;
        });

        payload_object.insert("model".to_string(), Value::String(route.model_id.clone()));
        let stream = payload_object
            .get("stream")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        tracing::debug!(stream, "processing stream option");

        let url = route.endpoint_url;
        tracing::debug!(%url, "target endpoint url");

        let request_body =
            serde_json::to_vec(&payload).map_err(|err| AppError::Internal(err.into()))?;

        tracing::debug!("sending request to upstream provider");
        let mut request_builder = self
            .http_client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(request_body.clone());

        match api_type {
            ApiType::AnthropicMessages => {
                request_builder = request_builder
                    .header("x-api-key", &route.provider_key.key)
                    .header("anthropic-version", "2023-06-01");
            }
            _ => {
                request_builder = request_builder.header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", route.provider_key.key),
                );
            }
        }

        let response = request_builder.send().await;

        let response = match response {
            Ok(response) => {
                tracing::debug!(status = ?response.status(), "received upstream response");

                if response.status() == reqwest::StatusCode::UNAUTHORIZED {
                    tracing::warn!(
                        provider_key_id = %route.provider_key.id,
                        provider_id = %route.provider_id,
                        "upstream returned 401 Unauthorized, disabling provider key"
                    );
                    let pool = self.pool.clone();
                    let key_id = route.provider_key.id;
                    tokio::spawn(async move {
                        if let Err(e) = provider_keys::update_key(
                            &pool,
                            key_id,
                            UpdateKeyParams {
                                name: None,
                                enabled: Some(false),
                            },
                        )
                        .await
                        {
                            tracing::error!(error = %e, key_id = %key_id, "failed to disable provider key");
                        }
                    });
                }

                response
            }
            Err(err) => {
                tracing::debug!(error = %err, "upstream request failed");
                let latency_ms = elapsed_ms(start);
                let context = RequestLogContext {
                    request_id,
                    gateway_key_id: Some(gateway_key_id.0),
                    api_type: Some(api_type),
                    model: Some(route.model_id.clone()),
                    alias: Some(route.alias_name),
                    provider: Some(route.provider_name.clone()),
                    endpoint: Some(url),
                    status_code: None,
                    latency_ms: Some(latency_ms),
                    client_ip: client_ip.clone(),
                    user_agent: user_agent.clone(),
                    request_body: Some(request_body),
                    response_body: Some(err.to_string().into_bytes()),
                    request_content_type: Some("application/json".to_string()),
                    response_content_type: Some("text/plain".to_string()),
                    prompt_tokens: None,
                    completion_tokens: None,
                    total_tokens: None,
                };
                let pool = self.pool.clone();
                tokio::spawn(async move {
                    if let Err(err) = logging::record_request(&pool, &context).await {
                        tracing::error!(error = %err, "failed to record request log");
                    }
                });
                return Err(AppError::Internal(err.into()));
            }
        };

        let status = response.status();
        let content_type = response.headers().get(header::CONTENT_TYPE).cloned();
        let response_content_type = content_type
            .as_ref()
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let model_id = route.model_id.clone();
        let provider_name = route.provider_name.clone();
        let endpoint = url.clone();
        let alias = route.alias_name.clone();

        let response = if stream {
            let (response_body_tx, response_body_rx) = tokio::sync::oneshot::channel::<Vec<u8>>();
            let mut stream_body = response.bytes_stream();
            let body_stream = async_stream::stream! {
                let mut captured = Vec::new();
                while let Some(next) = stream_body.next().await {
                    match next {
                        Ok(chunk) => {
                            captured.extend_from_slice(&chunk);
                            yield Ok::<_, std::convert::Infallible>(chunk);
                        }
                        Err(err) => {
                            tracing::error!(error = %err, "upstream stream error");
                            break;
                        }
                    }
                }
                let _ = response_body_tx.send(captured);
            };

            let mut builder = Response::builder().status(status);
            if let Some(value) = content_type.clone() {
                builder = builder.header(header::CONTENT_TYPE, value);
            }
            let response = builder
                .body(Body::from_stream(body_stream))
                .map_err(|err| AppError::Internal(err.into()))?;

            let pool = self.pool.clone();
            let request_body = request_body.clone();
            let response_content_type = response_content_type.clone();
            let client_ip = client_ip.clone();
            let user_agent = user_agent.clone();
            tokio::spawn(async move {
                let response_body = response_body_rx.await.ok();
                let (prompt_tokens, completion_tokens, total_tokens) = if let Some(body) = &response_body {
                    extract_usage(body, api_type)
                } else {
                    (None, None, None)
                };

                let context = RequestLogContext {
                    request_id,
                    gateway_key_id: Some(gateway_key_id.0),
                    api_type: Some(api_type),
                    model: Some(model_id),
                    alias: Some(alias),
                    provider: Some(provider_name),
                    endpoint: Some(endpoint),
                    status_code: Some(status.as_u16() as i32),
                    latency_ms: Some(elapsed_ms(start)),
                    client_ip,
                    user_agent,
                    request_body: Some(request_body),
                    response_body,
                    request_content_type: Some("application/json".to_string()),
                    response_content_type,
                    prompt_tokens,
                    completion_tokens,
                    total_tokens,
                };
                if let Err(err) = logging::record_request(&pool, &context).await {
                    tracing::error!(error = %err, "failed to record request log");
                }
            });

            response
        } else {
            let bytes = response
                .bytes()
                .await
                .map_err(|err| AppError::Internal(err.into()))?;
            let response_body = bytes.to_vec();
            let mut builder = Response::builder().status(status);
            if let Some(value) = content_type.clone() {
                builder = builder.header(header::CONTENT_TYPE, value);
            }
            let response = builder
                .body(Body::from(bytes))
                .map_err(|err| AppError::Internal(err.into()))?;

            let (prompt_tokens, completion_tokens, total_tokens) =
                extract_usage(&response_body, api_type);

            let context = RequestLogContext {
                request_id,
                gateway_key_id: Some(gateway_key_id.0),
                api_type: Some(api_type),
                model: Some(model_id),
                alias: Some(alias),
                provider: Some(provider_name),
                endpoint: Some(endpoint),
                status_code: Some(status.as_u16() as i32),
                latency_ms: Some(elapsed_ms(start)),
                client_ip,
                user_agent,
                request_body: Some(request_body),
                response_body: Some(response_body),
                request_content_type: Some("application/json".to_string()),
                response_content_type,
                prompt_tokens,
                completion_tokens,
                total_tokens,
            };
            let pool = self.pool.clone();
            tokio::spawn(async move {
                if let Err(err) = logging::record_request(&pool, &context).await {
                    tracing::error!(error = %err, "failed to record request log");
                }
            });

            response
        };

        Ok(response)
    }
}

fn elapsed_ms(start: Instant) -> i32 {
    i32::try_from(start.elapsed().as_millis()).unwrap_or(i32::MAX)
}

fn extract_usage(body: &[u8], api_type: ApiType) -> (Option<i32>, Option<i32>, Option<i32>) {
    // Try to parse as JSON first (non-streaming)
    if let Ok(json) = serde_json::from_slice::<Value>(body) {
        match api_type {
            ApiType::OpenAiChatCompletions | ApiType::OpenAiResponses => {
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
                 && let Some(usage) = json.get("usage") {
                    // Same extraction logic as above
                     match api_type {
                        ApiType::OpenAiChatCompletions | ApiType::OpenAiResponses => {
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
                             if let Some(input) = usage.get("input_tokens").and_then(Value::as_i64).map(|v| v as i32) {
                                let output = usage.get("output_tokens").and_then(Value::as_i64).map(|v| v as i32);
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