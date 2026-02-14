use axum::{
    Json,
    extract::{Query, State},
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{
    db::stats::{CategoryCount, TimeSeriesPoint},
    error::AppResult,
    services::stats,
    state::AppState,
};

#[derive(Deserialize)]
pub struct StatsQuery {
    #[serde(default)]
    #[serde(with = "time::serde::rfc3339::option")]
    pub start: Option<OffsetDateTime>,
    #[serde(default)]
    #[serde(with = "time::serde::rfc3339::option")]
    pub end: Option<OffsetDateTime>,
}

#[derive(Serialize)]
pub struct DashboardStats {
    pub requests_over_time: Vec<TimeSeriesPoint>,
    pub requests_by_provider: Vec<CategoryCount>,
}

pub async fn get_dashboard_stats(
    State(state): State<AppState>,
    Query(query): Query<StatsQuery>,
) -> AppResult<Json<DashboardStats>> {
    let end = query.end.unwrap_or_else(OffsetDateTime::now_utc);
    let start = query.start.unwrap_or_else(|| end - time::Duration::days(7));

    let requests_over_time = stats::get_requests_over_time(&state.pool, start, end).await?;
    let requests_by_provider = stats::get_requests_by_provider(&state.pool, start, end).await?;

    Ok(Json(DashboardStats {
        requests_over_time,
        requests_by_provider,
    }))
}
