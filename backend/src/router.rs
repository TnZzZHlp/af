use axum::{
    Router, middleware as axum_middleware,
    routing::post,
};

use crate::{handlers, middleware, state::AppState};

pub fn app(state: AppState) -> Router {
    let auth_routes = Router::new().route("/auth/login", post(handlers::auth::login));

    let protected_routes = Router::new()
        .route(
            "/v1/chat/completions",
            post(handlers::openai::chat_completions),
        )
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::rate_limit::rate_limit_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ));

    Router::new()
        .merge(auth_routes)
        .merge(protected_routes)
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::request_log::request_log_middleware,
        ))
        .with_state(state)
}
