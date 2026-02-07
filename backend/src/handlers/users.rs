use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time;

use crate::{
    error::{AppError, AppResult},
    services::users,
    state::AppState,
};

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub username: String,
    pub enabled: bool,
}

#[derive(Deserialize)]
pub struct UpdatePasswordRequest {
    pub password: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub enabled: bool,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: time::OffsetDateTime,
    #[serde(with = "time::serde::iso8601::option")]
    pub password_updated_at: Option<time::OffsetDateTime>,
}

impl From<users::User> for UserResponse {
    fn from(user: users::User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            enabled: user.enabled,
            created_at: user.created_at,
            password_updated_at: user.password_updated_at,
        }
    }
}

pub async fn list_users(State(state): State<AppState>) -> AppResult<Json<Vec<UserResponse>>> {
    let users = users::list_users(&state.pool)
        .await
        .map_err(AppError::Internal)?;
    Ok(Json(users.into_iter().map(Into::into).collect()))
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    let user = users::create_user(&state.pool, &payload.username, &payload.password)
        .await
        .map_err(AppError::Internal)?;
    Ok(Json(user.into()))
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<UserResponse>> {
    let user = users::get_user(&state.pool, id)
        .await
        .map_err(AppError::Internal)?
        .ok_or(AppError::NotFound)?;
    Ok(Json(user.into()))
}

pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    let user = users::update_user(&state.pool, id, &payload.username, payload.enabled)
        .await
        .map_err(AppError::Internal)?
        .ok_or(AppError::NotFound)?;
    Ok(Json(user.into()))
}

pub async fn update_password(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePasswordRequest>,
) -> AppResult<StatusCode> {
    users::change_password(&state.pool, id, &payload.password)
        .await
        .map_err(AppError::Internal)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    users::delete_user(&state.pool, id)
        .await
        .map_err(AppError::Internal)?;
    Ok(StatusCode::NO_CONTENT)
}
