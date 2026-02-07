use argon2::password_hash::{rand_core::OsRng, SaltString};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::gateway_keys::{self, GatewayKey};

pub async fn list_gateway_keys(
    pool: &PgPool,
    page: i64,
    page_size: i64,
) -> anyhow::Result<Vec<GatewayKey>> {
    let offset = (page - 1) * page_size;
    gateway_keys::list_gateway_keys(pool, page_size, offset).await
}

pub async fn create_gateway_key(
    pool: &PgPool,
    name: Option<String>,
    rate_limit_rps: Option<i32>,
    rate_limit_rpm: Option<i32>,
) -> anyhow::Result<GatewayKey> {
    let key = format!("sk-{}", SaltString::generate(&mut OsRng).as_str());

    let params = gateway_keys::CreateGatewayKeyParams {
        name,
        key,
        rate_limit_rps,
        rate_limit_rpm,
    };

    gateway_keys::create_gateway_key(pool, params).await
}

pub async fn get_gateway_key(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<GatewayKey>> {
    gateway_keys::fetch_gateway_key_by_id(pool, id).await
}

pub async fn update_gateway_key(
    pool: &PgPool,
    id: Uuid,
    name: Option<String>,
    enabled: Option<bool>,
    rate_limit_rps: Option<Option<i32>>,
    rate_limit_rpm: Option<Option<i32>>,
) -> anyhow::Result<Option<GatewayKey>> {
    let params = gateway_keys::UpdateGatewayKeyParams {
        name,
        enabled,
        rate_limit_rps,
        rate_limit_rpm,
    };

    gateway_keys::update_gateway_key(pool, id, params).await
}

pub async fn delete_gateway_key(pool: &PgPool, id: Uuid) -> anyhow::Result<bool> {
    gateway_keys::delete_gateway_key(pool, id).await
}
