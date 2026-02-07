use axum::{extract::{Query, State}, Json};
use serde::Deserialize;

use crate::{
    error::AppResult,
    services::logging::{self, RequestLogRow},
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
) -> AppResult<Json<Vec<RequestLogRow>>> {
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    let logs = logging::fetch_request_logs(&state.pool, limit, offset).await?;

    Ok(Json(logs))
}
