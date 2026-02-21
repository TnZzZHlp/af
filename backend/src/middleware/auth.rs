use axum::{
    body::{self, Body},
    extract::{ConnectInfo, State},
    http::{Request, header},
    middleware::Next,
    response::IntoResponse,
};
use serde_json::Value;
use std::net::SocketAddr;

use crate::{constants::MAX_REQUEST_BODY_BYTES, error::AppError, services::auth, state::AppState};
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub struct GatewayKeyId(pub Uuid);

pub async fn auth_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request<axum::body::Body>,
    next: Next,
) -> impl IntoResponse {
    let ip = addr.ip().to_string();

    if state.login_protection.is_banned(&ip).await {
        return AppError::Forbidden(
            "IP permanently banned due to excessive login failures".to_string(),
        )
        .into_response();
    }

    let api_key = auth::extract_api_key(req.headers());
    let Some(api_key) = api_key else {
        state.login_protection.record_failure(&ip).await;
        return AppError::Unauthorized.into_response();
    };

    let gateway_key = match auth::fetch_gateway_key(&state.pool, &api_key).await {
        Ok(Some(key)) => key,
        Ok(None) => {
            state.login_protection.record_failure(&ip).await;
            return AppError::Unauthorized.into_response();
        }
        Err(err) => return AppError::Internal(err).into_response(),
    };

    let mut req = req;
    req.extensions_mut().insert(GatewayKeyId(gateway_key.id));

    let whitelist = match auth::fetch_model_whitelist(&state.pool, gateway_key.id).await {
        Ok(models) => models,
        Err(err) => return AppError::Internal(err).into_response(),
    };
    if whitelist.is_empty() {
        return next.run(req).await;
    }

    let (parts, body) = req.into_parts();
    let body_bytes = match body::to_bytes(body, MAX_REQUEST_BODY_BYTES).await {
        Ok(bytes) => bytes,
        Err(_) => return AppError::BadRequest("invalid request body".to_string()).into_response(),
    };

    let model = match extract_model_for_whitelist_check(
        parts.uri.path(),
        parts.headers.get(header::CONTENT_TYPE),
        &body_bytes,
    ) {
        Ok(model) => model,
        Err(err) => return err.into_response(),
    };

    if whitelist.iter().all(|entry| entry != &model) {
        return AppError::Forbidden("model not in whitelist".to_string()).into_response();
    }

    let req = Request::from_parts(parts, Body::from(body_bytes));
    next.run(req).await
}

impl axum::extract::FromRequestParts<AppState> for GatewayKeyId {
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &AppState,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let id = parts.extensions.get::<GatewayKeyId>().copied();
        async move { id.ok_or(AppError::Unauthorized) }
    }
}

fn extract_model_for_whitelist_check(
    path: &str,
    content_type: Option<&header::HeaderValue>,
    body_bytes: &[u8],
) -> Result<String, AppError> {
    if !matches!(
        path,
        "/v1/chat/completions" | "/v1/responses" | "/v1/messages"
    ) {
        return Err(AppError::BadRequest("unsupported API path".to_string()));
    }

    let is_json = content_type
        .and_then(|value| value.to_str().ok())
        .map(|value| value.starts_with("application/json"))
        .unwrap_or(false);
    if !is_json {
        return Err(AppError::BadRequest(
            "content-type must be application/json".to_string(),
        ));
    }

    let payload: Value = serde_json::from_slice(body_bytes)
        .map_err(|_| AppError::BadRequest("invalid request body".to_string()))?;
    let payload_object = payload
        .as_object()
        .ok_or_else(|| AppError::BadRequest("payload must be a JSON object".to_string()))?;
    let model = payload_object
        .get("model")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::BadRequest("model is required".to_string()))?;

    Ok(model.to_string())
}
