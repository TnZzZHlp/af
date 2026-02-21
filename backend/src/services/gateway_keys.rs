use rand::RngExt;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::{
    gateway_key_models,
    gateway_keys::{self, GatewayKey},
};

pub async fn list_gateway_keys(
    pool: &PgPool,
    page: i64,
    page_size: i64,
) -> anyhow::Result<Vec<GatewayKey>> {
    let offset = (page - 1) * page_size;
    let mut keys = gateway_keys::list_gateway_keys(pool, page_size, offset).await?;

    for key in &mut keys {
        key.allowed_models = gateway_key_models::fetch_model_whitelist(pool, key.id).await?;
    }

    Ok(keys)
}

fn generate_random_key() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    const LENGTH: usize = 32;
    let mut rng = rand::rng();
    let key: String = (0..LENGTH)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    format!("sk-{}", key)
}

fn normalize_allowed_models(models: Option<Vec<String>>) -> Option<Vec<String>> {
    models.map(|models| {
        let mut normalized = Vec::new();
        for raw in models {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                continue;
            }
            if normalized.iter().any(|m| m == trimmed) {
                continue;
            }
            normalized.push(trimmed.to_string());
        }
        normalized
    })
}

pub async fn create_gateway_key(
    pool: &PgPool,
    name: Option<String>,
    rate_limit_rps: Option<i32>,
    rate_limit_rpm: Option<i32>,
    allowed_models: Option<Vec<String>>,
) -> anyhow::Result<GatewayKey> {
    let key = generate_random_key();

    let params = gateway_keys::CreateGatewayKeyParams {
        name,
        key,
        rate_limit_rps,
        rate_limit_rpm,
    };

    let mut created = gateway_keys::create_gateway_key(pool, params).await?;

    if let Some(models) = normalize_allowed_models(allowed_models) {
        gateway_key_models::replace_model_whitelist(pool, created.id, &models).await?;
        created.allowed_models = models;
    }

    Ok(created)
}

pub async fn get_gateway_key(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<GatewayKey>> {
    let mut key = match gateway_keys::fetch_gateway_key_by_id(pool, id).await? {
        Some(key) => key,
        None => return Ok(None),
    };

    key.allowed_models = gateway_key_models::fetch_model_whitelist(pool, key.id).await?;
    Ok(Some(key))
}

pub async fn update_gateway_key(
    pool: &PgPool,
    id: Uuid,
    name: Option<String>,
    enabled: Option<bool>,
    rate_limit_rps: Option<i32>,
    rate_limit_rpm: Option<i32>,
    allowed_models: Option<Vec<String>>,
) -> anyhow::Result<Option<GatewayKey>> {
    let params = gateway_keys::UpdateGatewayKeyParams {
        name,
        enabled,
        rate_limit_rps,
        rate_limit_rpm,
    };

    let mut key = match gateway_keys::update_gateway_key(pool, id, params).await? {
        Some(key) => key,
        None => return Ok(None),
    };

    if let Some(models) = normalize_allowed_models(allowed_models) {
        gateway_key_models::replace_model_whitelist(pool, key.id, &models).await?;
        key.allowed_models = models;
    } else {
        key.allowed_models = gateway_key_models::fetch_model_whitelist(pool, key.id).await?;
    }

    Ok(Some(key))
}

pub async fn delete_gateway_key(pool: &PgPool, id: Uuid) -> anyhow::Result<bool> {
    gateway_keys::delete_gateway_key(pool, id).await
}
