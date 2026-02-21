use axum::http::{HeaderMap, header};
use dashmap::DashMap;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use uuid::Uuid;

use crate::db::{
    gateway_key_models,
    gateway_keys::{self, GatewayKey},
};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone)]
struct IpRecord {
    failures: Vec<Instant>,
    banned: bool,
}

#[derive(Debug, Clone, Default)]
pub struct LoginProtection {
    records: Arc<DashMap<String, IpRecord>>,
}

impl LoginProtection {
    pub fn new() -> Self {
        Self {
            records: Arc::new(DashMap::new()),
        }
    }

    pub async fn is_banned(&self, ip: &str) -> bool {
        self.records
            .get(ip)
            .map(|record| record.banned)
            .unwrap_or(false)
    }

    pub async fn record_failure(&self, ip: &str) {
        let mut record = self.records.entry(ip.to_string()).or_insert(IpRecord {
            failures: Vec::new(),
            banned: false,
        });

        if record.banned {
            return;
        }

        let now = Instant::now();
        record.failures.push(now);

        // Remove old failures (> 1 minute)
        record
            .failures
            .retain(|&t| now.duration_since(t) <= Duration::from_secs(60));

        if record.failures.len() > 5 {
            record.banned = true;
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
}

pub fn create_jwt(user_id: Uuid, secret: &str) -> AppResult<String> {
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
    .map_err(|err| AppError::Internal(err.into()))
}

pub fn verify_jwt(token: &str, secret: &str) -> AppResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|err| AppError::Internal(err.into()))
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

pub async fn fetch_gateway_key(pool: &PgPool, api_key: &str) -> AppResult<Option<GatewayKey>> {
    let key = gateway_keys::fetch_gateway_key(pool, api_key).await?;

    Ok(key)
}

pub async fn fetch_model_whitelist(pool: &PgPool, gateway_key_id: Uuid) -> AppResult<Vec<String>> {
    gateway_key_models::fetch_model_whitelist(pool, gateway_key_id).await
}
