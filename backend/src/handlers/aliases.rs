use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    db::{
        aliases::AliasRow,
        alias_targets::{AliasTargetRow, AliasTargetDetail},
        types::ApiType,
    },
    error::{AppError, AppResult},
    services::aliases,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct ListAliasesQuery {
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

pub async fn list_aliases(
    State(state): State<AppState>,
    Query(query): Query<ListAliasesQuery>,
) -> AppResult<Json<Vec<AliasRow>>> {
    let aliases = aliases::list_aliases(&state.pool, query.page, query.page_size).await?;
    Ok(Json(aliases))
}

#[derive(Debug, Deserialize)]
pub struct CreateAliasRequest {
    pub name: String,
}

pub async fn create_alias(
    State(state): State<AppState>,
    Json(payload): Json<CreateAliasRequest>,
) -> AppResult<Json<AliasRow>> {
    let alias = aliases::create_alias(
        &state.pool,
        payload.name,
    )
    .await?;
    Ok(Json(alias))
}

pub async fn get_alias(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<AliasRow>> {
    let alias = aliases::get_alias(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(alias))
}

#[derive(Debug, Deserialize)]
pub struct UpdateAliasRequest {
    pub name: Option<String>,
    pub enabled: Option<bool>,
}

pub async fn update_alias(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateAliasRequest>,
) -> AppResult<Json<AliasRow>> {
    let alias = aliases::update_alias(
        &state.pool,
        id,
        payload.name,
        payload.enabled,
    )
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(alias))
}

pub async fn delete_alias(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<()> {
    if aliases::delete_alias(&state.pool, id).await? {
        Ok(())
    } else {
        Err(AppError::NotFound)
    }
}

// Alias Targets

pub async fn list_alias_targets(
    State(state): State<AppState>,
    Path(alias_id): Path<Uuid>,
) -> AppResult<Json<Vec<AliasTargetRow>>> {
    let targets = aliases::fetch_alias_targets(&state.pool, alias_id).await?;
    Ok(Json(targets))
}

pub async fn list_alias_target_details(
    State(state): State<AppState>,
    Path(alias_id): Path<Uuid>,
) -> AppResult<Json<Vec<AliasTargetDetail>>> {
    let details = aliases::fetch_all_alias_target_details(&state.pool, alias_id).await?;
    Ok(Json(details))
}

#[derive(Debug, Deserialize)]
pub struct CreateAliasTargetRequest {
    pub provider_id: Uuid,
    pub model_id: Uuid,
}

pub async fn create_alias_target(
    State(state): State<AppState>,
    Path(alias_id): Path<Uuid>,
    Json(payload): Json<CreateAliasTargetRequest>,
) -> AppResult<Json<AliasTargetRow>> {
    let target = aliases::create_alias_target(
        &state.pool,
        alias_id,
        payload.provider_id,
        payload.model_id,
    )
    .await?;
    Ok(Json(target))
}

#[derive(Debug, Deserialize)]
pub struct UpdateAliasTargetRequest {
    pub enabled: Option<bool>,
}

pub async fn update_alias_target(
    State(state): State<AppState>,
    Path((_alias_id, target_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateAliasTargetRequest>,
) -> AppResult<Json<AliasTargetRow>> {
    let target = aliases::update_alias_target(
        &state.pool,
        target_id,
        payload.enabled,
    )
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(target))
}

pub async fn delete_alias_target(
    State(state): State<AppState>,
    Path((_alias_id, target_id)): Path<(Uuid, Uuid)>,
) -> AppResult<()> {
    if aliases::delete_alias_target(&state.pool, target_id).await? {
        Ok(())
    } else {
        Err(AppError::NotFound)
    }
}
