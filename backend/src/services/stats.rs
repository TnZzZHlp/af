use sqlx::PgPool;
use time::OffsetDateTime;

use crate::db::stats::{self, CacheHitRateStats, CategoryCount, TimeSeriesPoint};
use crate::error::AppResult;

pub async fn get_requests_over_time(
    pool: &PgPool,
    start: OffsetDateTime,
    end: OffsetDateTime,
) -> AppResult<Vec<TimeSeriesPoint>> {
    let duration = end - start;
    let granularity_seconds = if duration <= time::Duration::days(1) {
        1800
    } else {
        21600
    };

    stats::requests_over_time(pool, start, end, granularity_seconds).await
}

pub async fn get_requests_by_provider(
    pool: &PgPool,
    start: OffsetDateTime,
    end: OffsetDateTime,
) -> AppResult<Vec<CategoryCount>> {
    stats::requests_by_provider(pool, start, end).await
}

pub async fn get_cache_hit_rate(
    pool: &PgPool,
    start: OffsetDateTime,
    end: OffsetDateTime,
) -> AppResult<CacheHitRateStats> {
    stats::cache_hit_rate(pool, start, end).await
}
