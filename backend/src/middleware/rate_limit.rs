use axum::{extract::State, http::Request, middleware::Next, response::IntoResponse};

use crate::{
    error::AppError, middleware::auth::GatewayKeyId, services::rate_limit, state::AppState,
};

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request<axum::body::Body>,
    next: Next,
) -> impl IntoResponse {
    let gateway_key_id = req.extensions().get::<GatewayKeyId>().copied();
    let Some(gateway_key_id) = gateway_key_id else {
        return AppError::Unauthorized.into_response();
    };

    let gateway_key_id = gateway_key_id.0;
    match rate_limit::fetch_limits(&state.pool, gateway_key_id).await {
        Ok((_rps, _rpm)) => {}
        Err(err) => return AppError::Internal(err).into_response(),
    }

    next.run(req).await
}
