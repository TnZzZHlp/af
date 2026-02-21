use crate::db::types::ApiType;
use crate::db::{
    provider_endpoints::{self, CreateEndpointParams, ProviderEndpoint, UpdateEndpointParams},
    provider_keys::{self, CreateKeyParams, ProviderKey, UpdateKeyParams},
    providers::{self, CreateProviderParams, Provider, UpdateProviderParams},
};
use crate::error::AppResult;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list_providers(pool: &PgPool, page: i64, page_size: i64) -> AppResult<Vec<Provider>> {
    providers::list_providers(pool, page, page_size).await
}

pub async fn create_provider(
    pool: &PgPool,
    name: String,
    description: Option<String>,
    brief: Option<String>,
) -> AppResult<Provider> {
    providers::create_provider(
        pool,
        CreateProviderParams {
            name,
            description,
            brief,
        },
    )
    .await
}

pub async fn get_provider(pool: &PgPool, id: Uuid) -> AppResult<Option<Provider>> {
    providers::fetch_provider_by_id(pool, id).await
}

pub async fn get_provider_by_brief(pool: &PgPool, brief: &str) -> AppResult<Option<Provider>> {
    providers::fetch_provider_by_brief(pool, brief).await
}

pub async fn update_provider(
    pool: &PgPool,
    id: Uuid,
    name: Option<String>,
    description: Option<String>,
    brief: Option<String>,
    enabled: Option<bool>,
) -> AppResult<Option<Provider>> {
    providers::update_provider(
        pool,
        id,
        UpdateProviderParams {
            name,
            description,
            brief,
            enabled,
        },
    )
    .await
}

pub async fn delete_provider(pool: &PgPool, id: Uuid) -> AppResult<bool> {
    providers::delete_provider(pool, id).await
}

// Endpoints

pub async fn list_endpoints_by_provider(
    pool: &PgPool,
    provider_id: Uuid,
) -> AppResult<Vec<ProviderEndpoint>> {
    provider_endpoints::list_endpoints_by_provider(pool, provider_id).await
}

pub async fn create_endpoint(
    pool: &PgPool,
    provider_id: Uuid,
    api_type: ApiType,
    url: String,
) -> AppResult<ProviderEndpoint> {
    provider_endpoints::create_endpoint(
        pool,
        CreateEndpointParams {
            provider_id,
            api_type,
            url,
        },
    )
    .await
}

pub async fn update_endpoint(
    pool: &PgPool,
    id: Uuid,
    url: Option<String>,
    enabled: Option<bool>,
) -> AppResult<Option<ProviderEndpoint>> {
    provider_endpoints::update_endpoint(pool, id, UpdateEndpointParams { url, enabled }).await
}

pub async fn delete_endpoint(pool: &PgPool, id: Uuid) -> AppResult<bool> {
    provider_endpoints::delete_endpoint(pool, id).await
}

pub async fn increment_usage_count(pool: &PgPool, id: Uuid) -> AppResult<()> {
    providers::increment_usage_count(pool, id).await
}

// Keys

pub async fn list_keys_by_provider(
    pool: &PgPool,
    provider_id: Uuid,
) -> AppResult<Vec<ProviderKey>> {
    provider_keys::list_keys_by_provider(pool, provider_id).await
}

pub async fn create_key(
    pool: &PgPool,
    provider_id: Uuid,
    name: Option<String>,
    key: String,
) -> AppResult<ProviderKey> {
    provider_keys::create_key(
        pool,
        CreateKeyParams {
            provider_id,
            name,
            key,
        },
    )
    .await
}

pub async fn update_key(
    pool: &PgPool,
    id: Uuid,
    name: Option<String>,
    enabled: Option<bool>,
) -> AppResult<Option<ProviderKey>> {
    provider_keys::update_key(pool, id, UpdateKeyParams { name, enabled }).await
}

pub async fn delete_key(pool: &PgPool, id: Uuid) -> AppResult<bool> {
    provider_keys::delete_key(pool, id).await
}
