use std::time::Instant;

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
    db::{
        alias_targets::AliasTargetDetail, provider_keys::ProviderKey,
        request_logs::RequestLogContext, types::ApiType,
    },
    error::{AppError, AppResult},
    middleware::{auth::GatewayKeyId, request_log::ClientInfo},
    services::{auth, logging, providers, routing},
};

use crate::db::provider_keys::{self, UpdateKeyParams};

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
        client_info: ClientInfo,
    ) -> AppResult<Response<Body>> {
        let request_id = Uuid::now_v7();
        let start = Instant::now();

        let ClientInfo {
            client_ip,
            user_agent,
        } = client_info;

        tracing::debug!(%request_id, "received chat completion request");

        let payload_object = payload
            .as_object_mut()
            .ok_or_else(|| AppError::BadRequest("payload must be a JSON object".to_string()))?;
        let model = payload_object
            .get("model")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| AppError::BadRequest("model is required".to_string()))?;

        tracing::debug!(%model, "extracted model from payload");

        let whitelist = auth::fetch_model_whitelist(&self.pool, gateway_key_id.0).await?;
        if !whitelist.is_empty() {
            tracing::debug!(?whitelist, "checking model whitelist");
            if !whitelist.iter().any(|entry| entry == &model) {
                tracing::debug!("model not in whitelist");
                return Err(AppError::Forbidden);
            }
        } else {
            tracing::debug!("whitelist is empty, skipping check");
        }

        tracing::debug!("fetching alias target details");
        let targets: Vec<AliasTargetDetail> =
            routing::fetch_alias_target_details(&self.pool, &model, ApiType::OpenAiChatCompletions)
                .await?;

        let target = targets
            .first()
            .ok_or_else(|| AppError::BadRequest(format!("unknown model alias: {model}")))?;

        tracing::debug!(
            provider_id = %target.provider_id,
            provider_name = %target.provider_name,
            target_model_id = %target.model_id,
            "resolved alias target"
        );

        let pool = self.pool.clone();
        let provider_id = target.provider_id;
        tokio::spawn(async move {
            let _ = providers::increment_usage_count(&pool, provider_id).await;
        });

        tracing::debug!("fetching provider keys");
        let provider_keys: Vec<ProviderKey> =
            routing::fetch_provider_keys(&self.pool, target.provider_id).await?;

        let provider_key = provider_keys
            .first()
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("no provider keys available")))?;

        tracing::debug!(provider_key_id = %provider_key.id, "selected provider key");

        let pool = self.pool.clone();
        let key_id = provider_key.id;
        tokio::spawn(async move {
            let _ = provider_keys::increment_usage_count(&pool, key_id).await;
        });

        payload_object.insert("model".to_string(), Value::String(target.model_id.clone()));
        let stream = payload_object
            .get("stream")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        tracing::debug!(stream, "processing stream option");

        let url = target.endpoint_url.clone().ok_or_else(|| {
            AppError::Internal(anyhow::anyhow!("no endpoint url found for target provider"))
        })?;

        tracing::debug!(%url, "target endpoint url");

        let request_body =
            serde_json::to_vec(&payload).map_err(|err| AppError::Internal(err.into()))?;

        tracing::debug!("sending request to upstream provider");
        let response = self
            .http_client
            .post(&url)
            .header(
                header::AUTHORIZATION,
                format!("Bearer {}", provider_key.key),
            )
            .header(header::CONTENT_TYPE, "application/json")
            .body(request_body.clone())
            .send()
            .await;

        let response = match response {
            Ok(response) => {
                tracing::debug!(status = ?response.status(), "received upstream response");

                if response.status() == reqwest::StatusCode::UNAUTHORIZED {
                    tracing::warn!(
                        provider_key_id = %provider_key.id,
                        provider_id = %target.provider_id,
                        "upstream returned 401 Unauthorized, disabling provider key"
                    );
                    let pool = self.pool.clone();
                    let key_id = provider_key.id;
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
                    api_type: Some(ApiType::OpenAiChatCompletions),
                    model: Some(target.model_id.clone()),
                    alias: Some(model),
                    provider: Some(target.provider_name.clone()),
                    endpoint: Some(url),
                    status_code: None,
                    latency_ms: Some(latency_ms),
                    client_ip: client_ip.clone(),
                    user_agent: user_agent.clone(),
                    request_body: Some(request_body),
                    response_body: Some(err.to_string().into_bytes()),
                    request_content_type: Some("application/json".to_string()),
                    response_content_type: Some("text/plain".to_string()),
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

        let model_id = target.model_id.clone();
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
            let response_content_type = response_content_type.clone();
            let client_ip = client_ip.clone();
            let user_agent = user_agent.clone();
            tokio::spawn(async move {
                let response_body = response_body_rx.await.ok();
                let context = RequestLogContext {
                    request_id,
                    gateway_key_id: Some(gateway_key_id.0),
                    api_type: Some(ApiType::OpenAiChatCompletions),
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

            let context = RequestLogContext {
                request_id,
                gateway_key_id: Some(gateway_key_id.0),
                api_type: Some(ApiType::OpenAiChatCompletions),
                model: Some(model_id),
                alias: Some(model),
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
