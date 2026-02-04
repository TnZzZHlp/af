use axum::{
    extract::State,
    http::{Request, header},
    middleware::Next,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{middleware::auth::GatewayKeyId, services::logging, state::AppState};

pub async fn request_log_middleware(
    State(state): State<AppState>,
    req: Request<axum::body::Body>,
    next: Next,
) -> impl IntoResponse {
    let request_id = Uuid::now_v7();
    let gateway_key_id = req.extensions().get::<GatewayKeyId>().copied();
    let user_agent = req
        .headers()
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);

    let context = logging::RequestLogContext {
        request_id,
        gateway_key_id: gateway_key_id.map(|id| id.0),
        api_type: None,
        model: None,
        alias: None,
        provider: None,
        endpoint: None,
        status_code: None,
        latency_ms: None,
        error_code: None,
        error_message: None,
        client_ip: None,
        user_agent,
        request_headers: None,
        request_body: None,
        response_headers: None,
        response_body: None,
    };

    let response = next.run(req).await;
    let _ = logging::record_request(&state.pool, &context).await;
    response
}
