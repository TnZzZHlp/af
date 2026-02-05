use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::{error::AppResult, services::users, state::AppState};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    let user = users::authenticate_user(&state.pool, payload.email.trim(), &payload.password)
        .await?
        .ok_or(crate::error::AppError::Unauthorized)?;

    let response = LoginResponse {
        id: user.id.to_string(),
        email: user.email,
        name: user.name,
    };

    Ok(Json(response))
}
