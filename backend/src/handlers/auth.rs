use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::{error::AppResult, services::users, state::AppState};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub id: String,
    pub username: String,
    pub token: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    let user = users::authenticate_user(&state.pool, payload.username.trim(), &payload.password)
        .await?
        .ok_or(crate::error::AppError::Unauthorized)?;

    let token = crate::services::auth::create_jwt(user.id, &state.jwt_secret)
        .map_err(crate::error::AppError::Internal)?;

    let response = LoginResponse {
        id: user.id.to_string(),
        username: user.username,
        token,
    };

    Ok(Json(response))
}
