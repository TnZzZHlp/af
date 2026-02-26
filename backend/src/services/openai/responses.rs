use std::time::Instant;

use axum::{
    body::Body,
    http::{Response, header},
};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    db::{
        provider_keys::{self, UpdateKeyParams},
        types::ApiType,
    },
    error::{AppError, AppResult},
    middleware::{auth::GatewayKeyId, request_log::ClientInfo},
    services::{logging, providers, routing},
    utils::extract_model_from_payload,
};

use super::{
    OpenAiService, streaming,
    utils::{RequestContext, extract_usage},
};

fn prepare_request_bodies(payload: Value, routed_model_id: &str, extra_fields: &Value) -> AppResult<(Vec<u8>, bool)> {
    let mut upstream_payload = payload;
    let payload_object = upstream_payload
        .as_object_mut()
        .ok_or_else(|| AppError::BadRequest("payload must be a JSON object".to_string()))?;
    payload_object.insert(
        "model".to_string(),
        Value::String(routed_model_id.to_string()),
    );
    
    if let Some(obj) = extra_fields.as_object() {
        for (k, v) in obj {
            payload_object.insert(k.clone(), v.clone());
        }
    }
    
    let stream = payload_object
        .get("stream")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let upstream_request_body =
        serde_json::to_vec(&upstream_payload).map_err(|err| AppError::Internal(err.into()))?;

    Ok((upstream_request_body, stream))
}

impl OpenAiService {
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

    pub(super) async fn process_request(
        &self,
        gateway_key_id: GatewayKeyId,
        payload: Value,
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

        let model = extract_model_from_payload(&payload)?;

        tracing::debug!(%model, "extracted model from payload");

        let route = routing::resolve_route(&self.pool, &model, api_type).await?;

        let pool = self.pool.clone();
        let provider_id = route.provider_id;
        let shutdown_token = self.background_tasks.token();
        self.background_tasks
            .spawn("providers.increment_usage_count", async move {
                if shutdown_token.is_cancelled() {
                    return;
                }
                let _ = providers::increment_usage_count(&pool, provider_id).await;
            });

        let pool = self.pool.clone();
        let key_id = route.provider_key.id;
        let shutdown_token = self.background_tasks.token();
        self.background_tasks
            .spawn("provider_keys.increment_usage_count", async move {
                if shutdown_token.is_cancelled() {
                    return;
                }
                let _ = provider_keys::increment_usage_count(&pool, key_id).await;
            });

        let (upstream_request_body, stream) = prepare_request_bodies(payload, &route.model_id, &route.extra_fields)?;

        tracing::debug!(stream, "processing stream option");

        let url = route.endpoint_url;
        tracing::debug!(%url, "target endpoint url");

        let request_context = RequestContext {
            request_id,
            gateway_key_id,
            api_type,
            model: route.model_id.clone(),
            alias: route.alias_name.clone(),
            provider: route.provider_name.clone(),
            endpoint: url.clone(),
            start,
            client_ip: client_ip.clone(),
            user_agent: user_agent.clone(),
            request_body: upstream_request_body.clone(),
        };

        tracing::debug!("sending request to upstream provider");
        let mut request_builder = self
            .http_client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(upstream_request_body.clone());

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
                    let shutdown_token = self.background_tasks.token();
                    self.background_tasks.spawn("provider_keys.disable_key", async move {
                        if shutdown_token.is_cancelled() {
                            return;
                        }
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
                let context = request_context.build_log_context(
                    None,
                    Some(err.to_string().into_bytes()),
                    Some("text/plain".to_string()),
                    None,
                    None,
                    None,
                );
                let pool = self.pool.clone();
                let shutdown_token = self.background_tasks.token();
                self.background_tasks
                    .spawn("request_log.record_failure", async move {
                        if shutdown_token.is_cancelled() {
                            return;
                        }
                        if let Err(log_err) = logging::record_request(&pool, &context).await {
                            tracing::error!(error = %log_err, "failed to record request log");
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

        let response = if stream {
            let (response, response_body_rx) =
                streaming::build_streaming_response(response, status, content_type.clone())?;

            let request_context = request_context.clone();
            let pool = self.pool.clone();
            let shutdown_token = self.background_tasks.token();
            self.background_tasks
                .spawn("request_log.record_stream_response", async move {
                    if shutdown_token.is_cancelled() {
                        return;
                    }
                    let response_body = response_body_rx.await.ok();
                    let (prompt_tokens, completion_tokens, total_tokens) =
                        if let Some(body) = &response_body {
                            extract_usage(body, request_context.api_type)
                        } else {
                            (None, None, None)
                        };

                    let context = request_context.build_log_context(
                        Some(status.as_u16() as i32),
                        response_body,
                        response_content_type,
                        prompt_tokens,
                        completion_tokens,
                        total_tokens,
                    );
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
            let (prompt_tokens, completion_tokens, total_tokens) =
                extract_usage(&response_body, request_context.api_type);

            let response = streaming::build_buffered_response(
                status,
                content_type.clone(),
                response_body.clone(),
            )?;

            let context = request_context.build_log_context(
                Some(status.as_u16() as i32),
                Some(response_body),
                response_content_type,
                prompt_tokens,
                completion_tokens,
                total_tokens,
            );
            let pool = self.pool.clone();
            let shutdown_token = self.background_tasks.token();
            self.background_tasks
                .spawn("request_log.record_response", async move {
                    if shutdown_token.is_cancelled() {
                        return;
                    }
                    if let Err(err) = logging::record_request(&pool, &context).await {
                        tracing::error!(error = %err, "failed to record request log");
                    }
                });

            response
        };

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::prepare_request_bodies;

    #[test]
    fn prepare_request_bodies_rewrites_model_for_upstream_body() {
        let payload = json!({
            "model": "alias-model",
            "input": "hello",
            "stream": true
        });

        let (upstream_request_body, stream) =
            prepare_request_bodies(payload, "provider-model", &json!({})).expect("request body should build");

        let upstream_json: Value = serde_json::from_slice(&upstream_request_body)
            .expect("upstream body should be valid JSON");

        assert!(stream);
        assert_eq!(
            upstream_json.get("model").and_then(Value::as_str),
            Some("provider-model")
        );
    }

    #[test]
    fn prepare_request_bodies_merges_extra_fields() {
        let payload = json!({
            "model": "alias-model",
            "messages": [{"role": "user", "content": "hello"}]
        });

        let extra_fields = json!({
            "enable_thinking": true,
            "temperature": 0.7
        });

        let (upstream_request_body, _) =
            prepare_request_bodies(payload, "provider-model", &extra_fields).expect("request body should build");

        let upstream_json: Value = serde_json::from_slice(&upstream_request_body)
            .expect("upstream body should be valid JSON");

        assert_eq!(
            upstream_json.get("model").and_then(Value::as_str),
            Some("provider-model")
        );
        assert_eq!(upstream_json.get("enable_thinking").and_then(Value::as_bool), Some(true));
        assert_eq!(upstream_json.get("temperature").and_then(Value::as_f64), Some(0.7));
    }

    #[test]
    fn prepare_request_bodies_overrides_existing_fields() {
        let payload = json!({
            "model": "alias-model",
            "temperature": 0.9,
            "enable_thinking": false
        });

        let extra_fields = json!({
            "enable_thinking": true
        });

        let (upstream_request_body, _) =
            prepare_request_bodies(payload, "provider-model", &extra_fields).expect("request body should build");

        let upstream_json: Value = serde_json::from_slice(&upstream_request_body)
            .expect("upstream body should be valid JSON");

        assert_eq!(upstream_json.get("enable_thinking").and_then(Value::as_bool), Some(true));
        assert_eq!(upstream_json.get("temperature").and_then(Value::as_f64), Some(0.9));
    }
}
