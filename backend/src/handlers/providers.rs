use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    db::{providers::ProviderRow, provider_endpoints::ProviderEndpointRow, provider_keys::ProviderKeyRow, types::ApiType},
    error::{AppError, AppResult},
    services::providers,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct ListProvidersQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    20
}

pub async fn list_providers(
    State(state): State<AppState>,
    Query(query): Query<ListProvidersQuery>,
) -> AppResult<Json<Vec<ProviderRow>>> {
    let providers = providers::list_providers(&state.pool, query.page, query.page_size).await?;
    Ok(Json(providers))
}

#[derive(Debug, Deserialize)]
pub struct CreateProviderRequest {
    pub name: String,
    pub description: Option<String>,
}

pub async fn create_provider(
    State(state): State<AppState>,
    Json(payload): Json<CreateProviderRequest>,
) -> AppResult<Json<ProviderRow>> {
    let provider = providers::create_provider(&state.pool, payload.name, payload.description).await?;
    Ok(Json(provider))
}

pub async fn get_provider(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ProviderRow>> {
    let provider = providers::get_provider(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(provider))
}

#[derive(Debug, Deserialize)]
pub struct UpdateProviderRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
}

pub async fn update_provider(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateProviderRequest>,
) -> AppResult<Json<ProviderRow>> {
    let provider = providers::update_provider(
        &state.pool,
        id,
        payload.name,
        payload.description,
        payload.enabled,
    )
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(provider))
}

pub async fn delete_provider(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<()> {
    if providers::delete_provider(&state.pool, id).await? {
        Ok(())
    } else {
        Err(AppError::NotFound)
    }
}

// Endpoints

pub async fn list_endpoints(
    State(state): State<AppState>,
    Path(provider_id): Path<Uuid>,
) -> AppResult<Json<Vec<ProviderEndpointRow>>> {
    let endpoints = providers::list_endpoints_by_provider(&state.pool, provider_id).await?;
    Ok(Json(endpoints))
}

#[derive(Debug, Deserialize)]
pub struct CreateEndpointRequest {
    pub api_type: ApiType,
    pub url: String,
    pub timeout_ms: Option<i32>,
}

pub async fn create_endpoint(
    State(state): State<AppState>,
    Path(provider_id): Path<Uuid>,
    Json(payload): Json<CreateEndpointRequest>,
) -> AppResult<Json<ProviderEndpointRow>> {
    let endpoint = providers::create_endpoint(
        &state.pool,
        provider_id,
        payload.api_type,
        payload.url,
        payload.timeout_ms,
    )
    .await?;
    Ok(Json(endpoint))
}

#[derive(Debug, Deserialize)]
pub struct UpdateEndpointRequest {
    pub url: Option<String>,
    pub timeout_ms: Option<i32>,
    pub enabled: Option<bool>,
}

pub async fn update_endpoint(
    State(state): State<AppState>,
    Path((_provider_id, id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateEndpointRequest>,
) -> AppResult<Json<ProviderEndpointRow>> {
    let endpoint = providers::update_endpoint(
        &state.pool,
        id,
        payload.url,
        payload.timeout_ms,
        payload.enabled,
    )
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(endpoint))
}

pub async fn delete_endpoint(
    State(state): State<AppState>,
    Path((_provider_id, id)): Path<(Uuid, Uuid)>,
) -> AppResult<()> {
    if providers::delete_endpoint(&state.pool, id).await? {
        Ok(())
    } else {
        Err(AppError::NotFound)
    }
}

// Keys

pub async fn list_keys(
    State(state): State<AppState>,
    Path(provider_id): Path<Uuid>,
) -> AppResult<Json<Vec<ProviderKeyRow>>> {
    let keys = providers::list_keys_by_provider(&state.pool, provider_id).await?;
    Ok(Json(keys))
}

#[derive(Debug, Deserialize)]
pub struct CreateKeyRequest {
    pub name: Option<String>,
    pub key: String,
    pub weight: Option<i32>,
}

pub async fn create_key(
    State(state): State<AppState>,
    Path(provider_id): Path<Uuid>,
    Json(payload): Json<CreateKeyRequest>,
) -> AppResult<Json<ProviderKeyRow>> {
    let key = providers::create_key(
        &state.pool,
        provider_id,
        payload.name,
        payload.key,
        payload.weight,
    )
    .await?;
    Ok(Json(key))
}

#[derive(Debug, Deserialize)]
pub struct UpdateKeyRequest {
    pub name: Option<String>,
    pub weight: Option<i32>,
    pub enabled: Option<bool>,
}

pub async fn update_key(
    State(state): State<AppState>,
    Path((_provider_id, id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateKeyRequest>,
) -> AppResult<Json<ProviderKeyRow>> {
    let key = providers::update_key(
        &state.pool,
        id,
        payload.name,
        payload.weight,
        payload.enabled,
    )
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(key))
}

pub async fn delete_key(
    State(state): State<AppState>,
    Path((_provider_id, id)): Path<(Uuid, Uuid)>,
) -> AppResult<()> {
    if providers::delete_key(&state.pool, id).await? {
        Ok(())
    } else {
        Err(AppError::NotFound)
    }
}
