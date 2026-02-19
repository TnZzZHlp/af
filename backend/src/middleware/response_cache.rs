use std::time::Instant;

use axum::{
    body::{self, Body},
    extract::State,
    http::{Request, Response, StatusCode, header},
    middleware::Next,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    db::{cache_log::CacheLogContext, types::ApiType},
    error::AppError,
    middleware::auth::GatewayKeyId,
    services::{
        logging,
        response_cache::{ResponseCacheKey, hash_request_body, request_body_hash_hex},
    },
    state::AppState,
};

const MAX_CACHEABLE_REQUEST_BODY_BYTES: usize = 100 * 1024 * 1024;

#[derive(Clone, Copy)]
enum CacheLayer {
    Moka,
    Database,
}

impl CacheLayer {
    fn as_str(self) -> &'static str {
        match self {
            Self::Moka => "moka",
            Self::Database => "database",
        }
    }
}

pub async fn response_cache_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    tracing::debug!("entering response cache middleware, checking for cacheable request");
    let Some(api_type) = api_type_from_path(req.uri().path()) else {
        return next.run(req).await;
    };

    let Some(gateway_key_id) = req.extensions().get::<GatewayKeyId>().copied() else {
        tracing::debug!("no gateway key id found in request extensions, skipping response cache");
        return next.run(req).await;
    };

    if !is_json_content_type(req.headers().get(header::CONTENT_TYPE)) {
        tracing::debug!("request content type is not JSON, skipping response cache");
        return next.run(req).await;
    }

    let start = Instant::now();
    let (parts, body) = req.into_parts();
    let request_bytes = match body::to_bytes(body, MAX_CACHEABLE_REQUEST_BODY_BYTES).await {
        Ok(bytes) => bytes,
        Err(err) => {
            tracing::warn!(error = %err, "failed to read request body for cache");
            return AppError::BadRequest("invalid request body".to_string()).into_response();
        }
    };

    if request_bytes.is_empty() {
        tracing::debug!("request body is empty, skipping response cache");
        let req = Request::from_parts(parts, Body::from(request_bytes));
        return next.run(req).await;
    }

    let request_body = request_bytes.to_vec();
    let cache_key = ResponseCacheKey::new(&request_body);

    if let Some(cached) = state.response_cache.get(&cache_key) {
        tracing::debug!(gateway_key_id = %gateway_key_id.0, %api_type, "response cache hit (moka)");
        spawn_cache_hit_log(
            &state,
            CacheHitLogArgs {
                gateway_key_id: gateway_key_id.0,
                api_type,
                request_body,
                cached: cached.clone(),
                latency_ms: elapsed_ms(start),
                cache_layer: CacheLayer::Moka,
            },
        );
        return build_cached_response(cached);
    }

    let request_body_hash = request_body_hash_hex(cache_key.request_body_hash);
    match logging::find_cached_response(&state.pool, &request_body_hash).await {
        Ok(Some(cached)) => {
            tracing::debug!(gateway_key_id = %gateway_key_id.0, %api_type, "response cache hit (database)");
            state.response_cache.insert(cache_key, cached.clone());
            spawn_cache_hit_log(
                &state,
                CacheHitLogArgs {
                    gateway_key_id: gateway_key_id.0,
                    api_type,
                    request_body,
                    cached: cached.clone(),
                    latency_ms: elapsed_ms(start),
                    cache_layer: CacheLayer::Database,
                },
            );
            build_cached_response(cached)
        }
        Ok(None) => {
            tracing::debug!(gateway_key_id = %gateway_key_id.0, %api_type, "response cache miss");
            let req = Request::from_parts(parts, Body::from(request_bytes));
            next.run(req).await
        }
        Err(err) => {
            tracing::error!(error = %err, "failed to query response cache from database");
            let req = Request::from_parts(parts, Body::from(request_bytes));
            next.run(req).await
        }
    }
}

fn build_cached_response(cached: crate::db::request_logs::CachedResponse) -> Response<Body> {
    let status = u16::try_from(cached.status_code)
        .ok()
        .and_then(|code| StatusCode::from_u16(code).ok())
        .unwrap_or(StatusCode::OK);
    let mut builder = Response::builder().status(status);
    if let Some(content_type) = cached.response_content_type {
        builder = builder.header(header::CONTENT_TYPE, content_type);
    }

    match builder.body(Body::from(cached.response_body)) {
        Ok(response) => response,
        Err(err) => {
            tracing::error!(error = %err, "failed to build cached response");
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("internal server error"))
                .expect("building static error response should not fail")
        }
    }
}

struct CacheHitLogArgs {
    gateway_key_id: Uuid,
    api_type: ApiType,
    request_body: Vec<u8>,
    cached: crate::db::request_logs::CachedResponse,
    latency_ms: i32,
    cache_layer: CacheLayer,
}

fn spawn_cache_hit_log(state: &AppState, args: CacheHitLogArgs) {
    let CacheHitLogArgs {
        gateway_key_id,
        api_type,
        request_body,
        cached,
        latency_ms,
        cache_layer,
    } = args;
    let pool = state.pool.clone();
    tokio::spawn(async move {
        let request_id = Uuid::now_v7();
        let request_body_hash = request_body_hash_hex(hash_request_body(&request_body));
        let request_body_size = i32::try_from(request_body.len()).unwrap_or(i32::MAX);
        let response_body_size = i32::try_from(cached.response_body.len()).unwrap_or(i32::MAX);

        let cache_context = CacheLogContext {
            request_id,
            source_request_log_id: Some(cached.source_request_log_id),
            gateway_key_id: Some(gateway_key_id),
            api_type,
            cache_layer: cache_layer.as_str(),
            hit: true,
            request_body_hash: Some(request_body_hash),
            request_body_size: Some(request_body_size),
            response_body_size: Some(response_body_size),
            latency_ms: Some(latency_ms),
        };
        if let Err(err) = logging::record_cache_event(&pool, &cache_context).await {
            tracing::error!(error = %err, "failed to record cache log");
        }
    });
}

fn api_type_from_path(path: &str) -> Option<ApiType> {
    match path {
        "/v1/chat/completions" => Some(ApiType::OpenAiChatCompletions),
        "/v1/responses" => Some(ApiType::OpenAiResponses),
        "/v1/messages" => Some(ApiType::AnthropicMessages),
        _ => None,
    }
}

fn is_json_content_type(content_type: Option<&header::HeaderValue>) -> bool {
    content_type
        .and_then(|value| value.to_str().ok())
        .map(|value| value.starts_with("application/json"))
        .unwrap_or(false)
}

fn elapsed_ms(start: Instant) -> i32 {
    i32::try_from(start.elapsed().as_millis()).unwrap_or(i32::MAX)
}
