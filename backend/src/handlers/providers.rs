use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    db::{
        provider_endpoints::ProviderEndpoint, provider_keys::ProviderKey, providers::Provider,
        types::ApiType,
    },
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
) -> AppResult<Json<Vec<Provider>>> {
    let providers = providers::list_providers(&state.pool, query.page, query.page_size).await?;
    Ok(Json(providers))
}

#[derive(Debug, Deserialize)]
pub struct CreateProviderRequest {
    pub name: String,
    pub description: Option<String>,
    pub brief: Option<String>,
}

pub async fn create_provider(
    State(state): State<AppState>,
    Json(payload): Json<CreateProviderRequest>,
) -> AppResult<Json<Provider>> {
    let provider = providers::create_provider(
        &state.pool,
        payload.name,
        payload.description,
        payload.brief,
    )
    .await?;
    Ok(Json(provider))
}

pub async fn get_provider(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Provider>> {
    let provider = providers::get_provider(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(provider))
}

#[derive(Debug, Deserialize)]
pub struct UpdateProviderRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub brief: Option<String>,
    pub enabled: Option<bool>,
}

pub async fn update_provider(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateProviderRequest>,
) -> AppResult<Json<Provider>> {
    let provider = providers::update_provider(
        &state.pool,
        id,
        payload.name,
        payload.description,
        payload.brief,
        payload.enabled,
    )
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(provider))
}

pub async fn delete_provider(State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<()> {
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
) -> AppResult<Json<Vec<ProviderEndpoint>>> {
    let endpoints = providers::list_endpoints_by_provider(&state.pool, provider_id).await?;
    Ok(Json(endpoints))
}

#[derive(Debug, Deserialize)]
pub struct CreateEndpointRequest {
    pub api_type: ApiType,
    pub url: String,
}

pub async fn create_endpoint(
    State(state): State<AppState>,
    Path(provider_id): Path<Uuid>,
    Json(payload): Json<CreateEndpointRequest>,
) -> AppResult<Json<ProviderEndpoint>> {
    let endpoint =
        providers::create_endpoint(&state.pool, provider_id, payload.api_type, payload.url).await?;
    Ok(Json(endpoint))
}

#[derive(Debug, Deserialize)]
pub struct UpdateEndpointRequest {
    pub url: Option<String>,
    pub enabled: Option<bool>,
}

pub async fn update_endpoint(
    State(state): State<AppState>,
    Path((_provider_id, id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateEndpointRequest>,
) -> AppResult<Json<ProviderEndpoint>> {
    let endpoint = providers::update_endpoint(&state.pool, id, payload.url, payload.enabled)
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
) -> AppResult<Json<Vec<ProviderKey>>> {
    let keys = providers::list_keys_by_provider(&state.pool, provider_id).await?;
    Ok(Json(keys))
}

#[derive(Debug, Deserialize)]
pub struct CreateKeyRequest {
    pub name: Option<String>,
    pub key: String,
}

pub async fn create_key(
    State(state): State<AppState>,
    Path(provider_id): Path<Uuid>,
    Json(payload): Json<CreateKeyRequest>,
) -> AppResult<Json<ProviderKey>> {
    let key = providers::create_key(&state.pool, provider_id, payload.name, payload.key).await?;
    Ok(Json(key))
}

#[derive(Debug, Deserialize)]
pub struct UpdateKeyRequest {
    pub name: Option<String>,
    pub enabled: Option<bool>,
}

pub async fn update_key(
    State(state): State<AppState>,
    Path((_provider_id, id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateKeyRequest>,
) -> AppResult<Json<ProviderKey>> {
    let key = providers::update_key(&state.pool, id, payload.name, payload.enabled)
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

// Models

pub async fn list_provider_models(
    State(state): State<AppState>,
    Path(provider_id): Path<Uuid>,
) -> AppResult<Json<Vec<crate::services::openai::Model>>> {
    let models = state.openai.list_models(provider_id).await?;
    Ok(Json(models))
}
