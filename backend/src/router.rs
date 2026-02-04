use axum::{middleware as axum_middleware, routing::get, Router};

use crate::{handlers, middleware, state::AppState};

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(handlers::health::healthz))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::rate_limit::rate_limit_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::request_log::request_log_middleware,
        ))
        .with_state(state)
}
