use std::net::IpAddr;

use axum::{
    extract::{ConnectInfo, State},
    http::{Request, header},
    middleware::Next,
    response::IntoResponse,
};

use crate::state::AppState;

#[derive(Clone, Debug)]
pub struct ClientInfo {
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
}

pub async fn request_log_middleware(
    State(_state): State<AppState>,
    req: Request<axum::body::Body>,
    next: Next,
) -> impl IntoResponse {
    let user_agent = req
        .headers()
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let client_ip = extract_client_ip(&req);

    let mut req = req;
    req.extensions_mut().insert(ClientInfo {
        client_ip: client_ip.clone(),
        user_agent: user_agent.clone(),
    });

    next.run(req).await
}

fn extract_client_ip(req: &Request<axum::body::Body>) -> Option<String> {
    if let Some(value) = req.headers().get("x-real-ip")
        && let Ok(ip_str) = value.to_str()
        && let Some(ip) = parse_ip(ip_str)
    {
        return Some(ip.to_string());
    }

    if let Some(value) = req.headers().get("x-forwarded-for")
        && let Ok(header_value) = value.to_str()
    {
        for part in header_value.split(',') {
            if let Some(ip) = parse_ip(part) {
                return Some(ip.to_string());
            }
        }
    }

    req.extensions()
        .get::<ConnectInfo<std::net::SocketAddr>>()
        .map(|info| info.0.ip().to_string())
}

fn parse_ip(value: &str) -> Option<IpAddr> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed.parse::<IpAddr>().ok()
}
