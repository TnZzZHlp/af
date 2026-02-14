use axum::{
    Json,
    extract::{ConnectInfo, State},
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    let ip = addr.ip().to_string();

    if state.login_protection.is_banned(&ip).await {
        return Err(crate::error::AppError::Forbidden(
            "IP permanently banned due to excessive login failures".to_string(),
        ));
    }

    let user =
        users::authenticate_user(&state.pool, payload.username.trim(), &payload.password).await?;

    let user = match user {
        Some(u) => u,
        None => {
            state.login_protection.record_failure(&ip).await;
            return Err(crate::error::AppError::Unauthorized);
        }
    };

    let token = crate::services::auth::create_jwt(user.id, &state.jwt_secret)
        .map_err(crate::error::AppError::Internal)?;

    let response = LoginResponse {
        id: user.id.to_string(),
        username: user.username,
        token,
    };

    Ok(Json(response))
}
