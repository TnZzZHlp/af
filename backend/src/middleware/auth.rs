use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};

use crate::{services::auth, state::AppState};
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub struct GatewayKeyId(pub Uuid);

pub async fn auth_middleware(
    State(state): State<AppState>,
    req: Request<axum::body::Body>,
    next: Next,
) -> impl IntoResponse {
    let api_key = auth::extract_api_key(req.headers());
    let Some(api_key) = api_key else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let gateway_key = match auth::fetch_gateway_key(&state.pool, &api_key).await {
        Ok(Some(key)) => key,
        Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let mut req = req;
    req.extensions_mut().insert(GatewayKeyId(gateway_key.id));

    next.run(req).await
}

impl axum::extract::FromRequestParts<AppState> for GatewayKeyId {
    type Rejection = StatusCode;

    fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &AppState,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let id = parts.extensions.get::<GatewayKeyId>().copied();
        async move { id.ok_or(StatusCode::UNAUTHORIZED) }
    }
}
