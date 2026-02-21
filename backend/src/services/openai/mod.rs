mod chat;
mod responses;
mod streaming;
mod utils;

use axum::http::header;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    db::types::ApiType,
    error::{AppError, AppResult},
    services::providers,
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

        let data = body.get("data").and_then(|v| v.as_array()).ok_or_else(|| {
            AppError::Internal(anyhow::anyhow!(
                "Invalid response format: 'data' field missing or not an array"
            ))
        })?;

        let provider_models: Vec<ProviderModel> =
            serde_json::from_value(Value::Array(data.clone()))
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
}
