use sqlx::PgPool;
use uuid::Uuid;
use crate::db::types::ApiType;
use crate::db::{
    providers::{self, ProviderRow, CreateProviderParams, UpdateProviderParams},
    provider_endpoints::{self, ProviderEndpointRow, CreateEndpointParams, UpdateEndpointParams},
    provider_keys::{self, ProviderKeyRow, CreateKeyParams, UpdateKeyParams},
};

pub async fn list_providers(pool: &PgPool, page: i64, page_size: i64) -> anyhow::Result<Vec<ProviderRow>> {
    providers::list_providers(pool, page, page_size).await
}

pub async fn create_provider(
    pool: &PgPool,
    name: String,
    description: Option<String>,
) -> anyhow::Result<ProviderRow> {
    providers::create_provider(pool, CreateProviderParams { name, description }).await
}

pub async fn get_provider(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<ProviderRow>> {
    providers::fetch_provider_by_id(pool, id).await
}

pub async fn update_provider(
    pool: &PgPool,
    id: Uuid,
    name: Option<String>,
    description: Option<String>,
    enabled: Option<bool>,
) -> anyhow::Result<Option<ProviderRow>> {
    providers::update_provider(pool, id, UpdateProviderParams { name, description, enabled }).await
}

pub async fn delete_provider(pool: &PgPool, id: Uuid) -> anyhow::Result<bool> {
    providers::delete_provider(pool, id).await
}

// Endpoints

pub async fn list_endpoints_by_provider(pool: &PgPool, provider_id: Uuid) -> anyhow::Result<Vec<ProviderEndpointRow>> {
    provider_endpoints::list_endpoints_by_provider(pool, provider_id).await
}

pub async fn create_endpoint(
    pool: &PgPool,
    provider_id: Uuid,
    api_type: ApiType,
    url: String,
    weight: Option<i32>,
    priority: Option<i32>,
    timeout_ms: Option<i32>,
) -> anyhow::Result<ProviderEndpointRow> {
    provider_endpoints::create_endpoint(pool, CreateEndpointParams {
        provider_id,
        api_type,
        url,
        weight,
        priority,
        timeout_ms,
    }).await
}

pub async fn update_endpoint(
    pool: &PgPool,
    id: Uuid,
    url: Option<String>,
    weight: Option<i32>,
    priority: Option<i32>,
    timeout_ms: Option<i32>,
    enabled: Option<bool>,
) -> anyhow::Result<Option<ProviderEndpointRow>> {
    provider_endpoints::update_endpoint(pool, id, UpdateEndpointParams {
        url,
        weight,
        priority,
        timeout_ms,
        enabled,
    }).await
}

pub async fn delete_endpoint(pool: &PgPool, id: Uuid) -> anyhow::Result<bool> {
    provider_endpoints::delete_endpoint(pool, id).await
}

// Keys

pub async fn list_keys_by_provider(pool: &PgPool, provider_id: Uuid) -> anyhow::Result<Vec<ProviderKeyRow>> {
    provider_keys::list_keys_by_provider(pool, provider_id).await
}

pub async fn create_key(
    pool: &PgPool,
    provider_id: Uuid,
    name: Option<String>,
    key: String,
    weight: Option<i32>,
) -> anyhow::Result<ProviderKeyRow> {
    provider_keys::create_key(pool, CreateKeyParams {
        provider_id,
        name,
        key,
        weight,
    }).await
}

pub async fn update_key(
    pool: &PgPool,
    id: Uuid,
    name: Option<String>,
    weight: Option<i32>,
    enabled: Option<bool>,
) -> anyhow::Result<Option<ProviderKeyRow>> {
    provider_keys::update_key(pool, id, UpdateKeyParams {
        name,
        weight,
        enabled,
    }).await
}

pub async fn delete_key(pool: &PgPool, id: Uuid) -> anyhow::Result<bool> {
    provider_keys::delete_key(pool, id).await
}
