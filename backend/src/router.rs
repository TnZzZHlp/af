use axum::{
    Router, middleware as axum_middleware,
    routing::{get, post, put},
};
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer,
};
use tracing::Level;

use crate::{handlers, middleware, state::AppState};

pub fn app(state: AppState) -> Router {
    let auth_routes = Router::new().route("/auth/login", post(handlers::auth::login));

    let gateway_key_routes = Router::new()
        .route(
            "/gateway-keys",
            get(handlers::gateway_keys::list).post(handlers::gateway_keys::create),
        )
        .route(
            "/gateway-keys/{id}",
            get(handlers::gateway_keys::get)
                .put(handlers::gateway_keys::update)
                .delete(handlers::gateway_keys::delete),
        );

    let provider_routes = Router::new()
        .route(
            "/providers",
            get(handlers::providers::list_providers).post(handlers::providers::create_provider),
        )
        .route(
            "/providers/{id}",
            get(handlers::providers::get_provider)
                .put(handlers::providers::update_provider)
                .delete(handlers::providers::delete_provider),
        )
        .route(
            "/providers/{id}/endpoints",
            get(handlers::providers::list_endpoints).post(handlers::providers::create_endpoint),
        )
        .route(
            "/providers/{id}/endpoints/{endpoint_id}",
            put(handlers::providers::update_endpoint).delete(handlers::providers::delete_endpoint),
        )
        .route(
            "/providers/{id}/keys",
            get(handlers::providers::list_keys).post(handlers::providers::create_key),
        )
        .route(
            "/providers/{id}/keys/{key_id}",
            put(handlers::providers::update_key).delete(handlers::providers::delete_key),
        );

    let alias_routes = Router::new()
        .route(
            "/aliases",
            get(handlers::aliases::list_aliases).post(handlers::aliases::create_alias),
        )
        .route(
            "/aliases/{id}",
            get(handlers::aliases::get_alias)
                .put(handlers::aliases::update_alias)
                .delete(handlers::aliases::delete_alias),
        )
        .route(
            "/aliases/{id}/targets/details",
            get(handlers::aliases::list_alias_target_details),
        )
        .route(
            "/aliases/{id}/targets",
            post(handlers::aliases::create_alias_target),
        )
        .route(
            "/aliases/{id}/targets/{target_id}",
            put(handlers::aliases::update_alias_target)
                .delete(handlers::aliases::delete_alias_target),
        );

    let request_log_routes = Router::new()
        .route(
            "/request-logs",
            get(handlers::request_logs::list_request_logs),
        )
        .route(
            "/request-logs/{id}",
            get(handlers::request_logs::get_request_log),
        );

    let user_routes = Router::new()
        .route(
            "/users",
            get(handlers::users::list_users).post(handlers::users::create_user),
        )
        .route(
            "/users/{id}",
            get(handlers::users::get_user)
                .put(handlers::users::update_user)
                .delete(handlers::users::delete_user),
        )
        .route(
            "/users/{id}/password",
            put(handlers::users::update_password),
        );

    let ai_routes = Router::new()
        .route(
            "/v1/chat/completions",
            post(handlers::openai::chat_completions),
        )
        .route(
            "/v1/responses",
            post(handlers::openai::responses),
        )
        .route(
            "/v1/messages",
            post(handlers::openai::anthropic_messages),
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
            Router::new().merge(auth_routes).merge(
                Router::new()
                    .merge(gateway_key_routes)
                    .merge(provider_routes)
                    .merge(alias_routes)
                    .merge(request_log_routes)
                    .merge(user_routes)
                    .layer(axum_middleware::from_fn_with_state(
                        state.clone(),
                        middleware::admin_auth::admin_auth_middleware,
                    )),
            ),
        )
        .merge(ai_routes)
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::request_log::request_log_middleware,
        ))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    DefaultMakeSpan::new()
                        .include_headers(true)
                        .level(Level::INFO),
                )
                .on_request(DefaultOnRequest::new().level(Level::DEBUG))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
                .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
        )
        .with_state(state)
}
