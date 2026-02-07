use axum::{Extension, Json, body::Body, extract::State, http::Response};
use serde_json::Value;

use crate::{
    error::AppResult,
    middleware::{auth::GatewayKeyId, request_log::ClientInfo},
    state::AppState,
};

pub async fn chat_completions(
    State(state): State<AppState>,
    gateway_key_id: GatewayKeyId,
    Extension(client_info): Extension<ClientInfo>,
    Json(payload): Json<Value>,
) -> AppResult<Response<Body>> {
    state
        .openai
        .chat_completions(gateway_key_id, payload, client_info)
        .await
}
