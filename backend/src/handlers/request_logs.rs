use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    services::logging::{self, RequestLog, RequestLogSummary},
    state::AppState,
};

#[derive(Deserialize)]
pub struct ListRequestLogsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_request_logs(
    State(state): State<AppState>,
    Query(query): Query<ListRequestLogsQuery>,
) -> AppResult<Json<Vec<RequestLogSummary>>> {
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    let logs = logging::fetch_request_logs(&state.pool, limit, offset).await?;

    Ok(Json(logs))
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
