use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    db::gateway_keys::GatewayKey,
    error::{AppError, AppResult},
    services::gateway_keys,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct ListGatewayKeysQuery {
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

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<ListGatewayKeysQuery>,
) -> AppResult<Json<Vec<GatewayKey>>> {
    let keys = gateway_keys::list_gateway_keys(&state.pool, query.page, query.page_size).await?;
    Ok(Json(keys))
}

#[derive(Debug, Deserialize)]
pub struct CreateGatewayKeyRequest {
    pub name: Option<String>,
    pub rate_limit_rps: Option<i32>,
    pub rate_limit_rpm: Option<i32>,
    pub allowed_models: Option<Vec<String>>,
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateGatewayKeyRequest>,
) -> AppResult<Json<GatewayKey>> {
    let key = gateway_keys::create_gateway_key(
        &state.pool,
        payload.name,
        payload.rate_limit_rps,
        payload.rate_limit_rpm,
        payload.allowed_models,
    )
    .await?;
    Ok(Json(key))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<GatewayKey>> {
    let key = gateway_keys::get_gateway_key(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(key))
}

#[derive(Debug, Deserialize)]
pub struct UpdateGatewayKeyRequest {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub rate_limit_rps: Option<i32>,
    pub rate_limit_rpm: Option<i32>,
    pub allowed_models: Option<Vec<String>>,
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateGatewayKeyRequest>,
) -> AppResult<Json<GatewayKey>> {
    let key = gateway_keys::update_gateway_key(
        &state.pool,
        id,
        payload.name,
        payload.enabled,
        payload.rate_limit_rps,
        payload.rate_limit_rpm,
        payload.allowed_models,
    )
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(key))
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<()> {
    if gateway_keys::delete_gateway_key(&state.pool, id).await? {
        Ok(())
    } else {
        Err(AppError::NotFound)
    }
}
