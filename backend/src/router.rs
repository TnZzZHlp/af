use axum::{
    Router, middleware as axum_middleware,
    routing::{delete, get, post, put},
};

use crate::{handlers, middleware, state::AppState};

pub fn app(state: AppState) -> Router {
    let auth_routes = Router::new().route("/auth/login", post(handlers::auth::login));

    let gateway_key_routes = Router::new()
        .route("/gateway-keys", get(handlers::gateway_keys::list))
        .route("/gateway-keys", post(handlers::gateway_keys::create))
        .route("/gateway-keys/{id}", get(handlers::gateway_keys::get))
        .route("/gateway-keys/{id}", put(handlers::gateway_keys::update))
        .route("/gateway-keys/{id}", delete(handlers::gateway_keys::delete));

    let provider_routes = Router::new()
        .route("/providers", get(handlers::providers::list_providers))
        .route("/providers", post(handlers::providers::create_provider))
        .route("/providers/{id}", get(handlers::providers::get_provider))
        .route("/providers/{id}", put(handlers::providers::update_provider))
        .route(
            "/providers/{id}",
            delete(handlers::providers::delete_provider),
        )
        .route(
            "/providers/{id}/endpoints",
            get(handlers::providers::list_endpoints),
        )
        .route(
            "/providers/{id}/endpoints",
            post(handlers::providers::create_endpoint),
        )
        .route(
            "/providers/{id}/endpoints/{endpoint_id}",
            put(handlers::providers::update_endpoint),
        )
        .route(
            "/providers/{id}/endpoints/{endpoint_id}",
            delete(handlers::providers::delete_endpoint),
        )
        .route("/providers/{id}/keys", get(handlers::providers::list_keys))
        .route(
            "/providers/{id}/keys",
            post(handlers::providers::create_key),
        )
        .route(
            "/providers/{id}/keys/{key_id}",
            put(handlers::providers::update_key),
        )
        .route(
            "/providers/{id}/keys/{key_id}",
            delete(handlers::providers::delete_key),
        );

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
        .nest(
            "/api",
            Router::new()
                .merge(auth_routes)
                .merge(
                    Router::new()
                        .merge(gateway_key_routes)
                        .merge(provider_routes)
                        .layer(axum_middleware::from_fn_with_state(
                            state.clone(),
                            middleware::admin_auth::admin_auth_middleware,
                        )),
                )
                .merge(protected_routes),
        )
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::request_log::request_log_middleware,
        ))
        .with_state(state)
}
