use std::time::{Duration, Instant};

use axum::{
    body::Body,
    http::{Response, header},
};
use futures_util::StreamExt;
use reqwest::Client;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    middleware::auth::GatewayKeyId,
    services::{auth, logging, providers, routing},
};

#[derive(Clone)]
pub struct OpenAiService {
    pool: PgPool,
    http_client: Client,
}

impl OpenAiService {
    pub fn new(pool: PgPool, http_client: Client) -> Self {
        Self { pool, http_client }
    }

    pub async fn chat_completions(
        &self,
        gateway_key_id: GatewayKeyId,
        mut payload: Value,
    ) -> AppResult<Response<Body>> {
        let request_id = Uuid::now_v7();
        let start = Instant::now();

        let payload_object = payload
            .as_object_mut()
            .ok_or_else(|| AppError::BadRequest("payload must be a JSON object".to_string()))?;
        let model = payload_object
            .get("model")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| AppError::BadRequest("model is required".to_string()))?;

        let whitelist = auth::fetch_model_whitelist(&self.pool, gateway_key_id.0).await?;
        if !whitelist.is_empty() && !whitelist.iter().any(|entry| entry == &model) {
            return Err(AppError::Forbidden);
        }

        let mut targets =
            routing::fetch_alias_target_details(&self.pool, &model, logging::ApiType::OpenAiChatCompletions)
                .await?;
        let target = targets
            .pop()
            .ok_or_else(|| AppError::BadRequest(format!("unknown model alias: {model}")))?;

        let _ = providers::increment_usage_count(&self.pool, target.provider_id).await;

        let mut provider_keys =
            routing::fetch_provider_keys(&self.pool, target.provider_id).await?;
        let provider_key = provider_keys
            .pop()
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("no provider keys available")))?;

        payload_object.insert(
            "model".to_string(),
            Value::String(target.model_name.clone()),
        );
        let stream = payload_object
            .get("stream")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let url = target.endpoint_url.clone();

        let request_body =
            serde_json::to_vec(&payload).map_err(|err| AppError::Internal(err.into()))?;
        let response = self
            .http_client
            .post(&url)
            .header(
                header::AUTHORIZATION,
                format!("Bearer {}", provider_key.key),
            )
            .header(header::CONTENT_TYPE, "application/json")
            .timeout(Duration::from_millis(target.endpoint_timeout_ms as u64))
            .body(request_body.clone())
            .send()
            .await;

        let response = match response {
            Ok(response) => response,
            Err(err) => {
                let latency_ms = elapsed_ms(start);
                let context = logging::RequestLogContext {
                    request_id,
                    gateway_key_id: Some(gateway_key_id.0),
                    api_type: Some(logging::ApiType::OpenAiChatCompletions),
                    model: Some(target.model_name),
                    alias: Some(model),
                    provider: Some(target.provider_name),
                    endpoint: Some(url),
                    status_code: None,
                    latency_ms: Some(latency_ms),
                    client_ip: None,
                    user_agent: None,
                    request_body: Some(request_body),
                    response_body: Some(err.to_string().into_bytes()),
                };
                let _ = logging::record_request(&self.pool, &context).await;
                return Err(AppError::Internal(err.into()));
            }
        };

        let status = response.status();
        let content_type = response.headers().get(header::CONTENT_TYPE).cloned();

        let model_name = target.model_name.clone();
        let provider_name = target.provider_name.clone();
        let endpoint = url.clone();

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
            let alias = model.clone();
            let request_body = request_body.clone();
            tokio::spawn(async move {
                let response_body = response_body_rx.await.ok();
                let context = logging::RequestLogContext {
                    request_id,
                    gateway_key_id: Some(gateway_key_id.0),
                    api_type: Some(logging::ApiType::OpenAiChatCompletions),
                    model: Some(model_name),
                    alias: Some(alias),
                    provider: Some(provider_name),
                    endpoint: Some(endpoint),
                    status_code: Some(status.as_u16() as i32),
                    latency_ms: Some(elapsed_ms(start)),
                    client_ip: None,
                    user_agent: None,
                    request_body: Some(request_body),
                    response_body,
                };
                let _ = logging::record_request(&pool, &context).await;
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

            let context = logging::RequestLogContext {
                request_id,
                gateway_key_id: Some(gateway_key_id.0),
                api_type: Some(logging::ApiType::OpenAiChatCompletions),
                model: Some(model_name),
                alias: Some(model),
                provider: Some(provider_name),
                endpoint: Some(endpoint),
                status_code: Some(status.as_u16() as i32),
                latency_ms: Some(elapsed_ms(start)),
                client_ip: None,
                user_agent: None,
                request_body: Some(request_body),
                response_body: Some(response_body),
            };
            let _ = logging::record_request(&self.pool, &context).await;

            response
        };

        Ok(response)
    }
}

fn elapsed_ms(start: Instant) -> i32 {
    i32::try_from(start.elapsed().as_millis()).unwrap_or(i32::MAX)
}
