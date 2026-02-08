use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    db::request_logs::{RequestLog, RequestLogFilter, RequestLogSummary},
    error::{AppError, AppResult},
    services::logging::{self},
    state::AppState,
};

#[derive(Deserialize)]
pub struct ListRequestLogsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub request_id: Option<Uuid>,
    pub model: Option<String>,
    pub alias: Option<String>,
    pub provider: Option<String>,
    pub status_code: Option<i32>,
    pub client_ip: Option<String>,
}

#[derive(Serialize)]
pub struct ListRequestLogsResponse {
    pub data: Vec<RequestLogSummary>,
    pub total: i64,
}

pub async fn list_request_logs(
    State(state): State<AppState>,
    Query(query): Query<ListRequestLogsQuery>,
) -> AppResult<Json<ListRequestLogsResponse>> {
    let filter = RequestLogFilter {
        limit: query.limit.or(Some(20)),
        offset: query.offset.or(Some(0)),
        request_id: query.request_id,
        model: query.model,
        alias: query.alias,
        provider: query.provider,
        status_code: query.status_code,
        client_ip: query.client_ip,
    };

    let logs = logging::fetch_request_logs(&state.pool, &filter).await?;
    let total = logging::count_request_logs(&state.pool, &filter).await?;

    Ok(Json(ListRequestLogsResponse { data: logs, total }))
}

pub async fn get_request_log(
    State(state): State<AppState>,
    Path(request_id): Path<Uuid>,
) -> AppResult<Json<RequestLog>> {
    let log = logging::fetch_request_log_detail(&state.pool, request_id).await?;

    match log {
        Some(log) => Ok(Json(log)),
        None => Err(AppError::NotFound),
    }
}
