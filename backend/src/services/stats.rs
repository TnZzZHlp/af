use sqlx::PgPool;
use time::OffsetDateTime;

use crate::db::stats::{self, CategoryCount, TimeSeriesPoint};

pub async fn get_requests_over_time(
    pool: &PgPool,
    start: OffsetDateTime,
    end: OffsetDateTime,
) -> anyhow::Result<Vec<TimeSeriesPoint>> {
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
) -> anyhow::Result<Vec<CategoryCount>> {
    stats::requests_by_provider(pool, start, end).await
}
