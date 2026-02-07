use axum::http::{HeaderMap, header};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::{gateway_key_models, gateway_keys};

#[derive(Debug, Clone)]
pub struct GatewayKey {
    pub id: Uuid,
    pub name: Option<String>,
    pub rate_limit_rps: Option<i32>,
    pub rate_limit_rpm: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
}

pub fn create_jwt(user_id: Uuid, secret: &str) -> anyhow::Result<String> {
    let expiration = time::OffsetDateTime::now_utc() + time::Duration::days(7);
    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration.unix_timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|err| anyhow::anyhow!(err))
}

pub fn verify_jwt(token: &str, secret: &str) -> anyhow::Result<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|err| anyhow::anyhow!(err))
}

pub fn extract_api_key(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::to_string)
        .or_else(|| {
            headers
                .get("x-api-key")
                .and_then(|value| value.to_str().ok())
                .map(str::to_string)
        })
}

pub async fn fetch_gateway_key(pool: &PgPool, api_key: &str) -> anyhow::Result<Option<GatewayKey>> {
    let key = match gateway_keys::fetch_gateway_key(pool, api_key).await? {
        Some(key) => key,
        None => return Ok(None),
    };

    Ok(Some(GatewayKey {
        id: key.id,
        name: key.name,
        rate_limit_rps: key.rate_limit_rps,
        rate_limit_rpm: key.rate_limit_rpm,
    }))
}

pub async fn fetch_model_whitelist(
    pool: &PgPool,
    gateway_key_id: Uuid,
) -> anyhow::Result<Vec<String>> {
    gateway_key_models::fetch_model_whitelist(pool, gateway_key_id).await
}
