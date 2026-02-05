use axum::{
    extract::State,
    http::Request,
    middleware::Next,
    response::IntoResponse,
};

use crate::{error::AppError, services::auth, state::AppState};
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
        return AppError::Unauthorized.into_response();
    };

    let gateway_key = match auth::fetch_gateway_key(&state.pool, &api_key).await {
        Ok(Some(key)) => key,
        Ok(None) => return AppError::Unauthorized.into_response(),
        Err(err) => return AppError::Internal(err).into_response(),
    };

    let mut req = req;
    req.extensions_mut().insert(GatewayKeyId(gateway_key.id));

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
