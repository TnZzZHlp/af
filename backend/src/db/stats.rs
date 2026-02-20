use serde::Serialize;
use sqlx::PgPool;
use time::OffsetDateTime;

#[derive(Serialize, sqlx::FromRow)]
pub struct TimeSeriesPoint {
    #[serde(with = "time::serde::rfc3339")]
    pub time: OffsetDateTime,
    pub count: i64,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct CategoryCount {
    pub category: Option<String>,
    pub count: i64,
}

#[derive(sqlx::FromRow)]
pub struct CacheHitRateStats {
    pub cache_hit_requests: i64,
    pub cache_total_requests: i64,
    pub cache_hit_rate: f64,
}

pub async fn requests_over_time(
    pool: &PgPool,
    start: OffsetDateTime,
    end: OffsetDateTime,
    granularity_seconds: i64,
) -> anyhow::Result<Vec<TimeSeriesPoint>> {
    let rows = sqlx::query_as!(
        TimeSeriesPoint,
        r#"
        SELECT
            to_timestamp(floor(extract(epoch from created_at) / $3) * $3) as "time!",
            count(*) as "count!"
        FROM request_logs
        WHERE created_at >= $1 AND created_at <= $2
        GROUP BY 1
        ORDER BY 1 ASC
        "#,
        start,
        end,
        granularity_seconds as f64
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn requests_by_provider(
    pool: &PgPool,
    start: OffsetDateTime,
    end: OffsetDateTime,
) -> anyhow::Result<Vec<CategoryCount>> {
    let rows = sqlx::query_as!(
        CategoryCount,
        r#"
        SELECT
            provider as category,
            count(*) as "count!"
        FROM request_logs
        WHERE created_at >= $1 AND created_at <= $2
        GROUP BY provider
        ORDER BY "count!" DESC
        "#,
        start,
        end
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn cache_hit_rate(
    pool: &PgPool,
    start: OffsetDateTime,
    end: OffsetDateTime,
) -> anyhow::Result<CacheHitRateStats> {
    let stats = sqlx::query_as::<_, CacheHitRateStats>(
        r#"
        WITH request_count AS (
            SELECT count(*)::bigint AS total
            FROM request_logs
            WHERE created_at >= $1 AND created_at <= $2
        ),
        cache_hit_count AS (
            SELECT count(*)::bigint AS total
            FROM cache_log
            WHERE created_at >= $1 AND created_at <= $2
        )
        SELECT
            chc.total AS cache_hit_requests,
            (rc.total + chc.total) AS cache_total_requests,
            CASE
                WHEN (rc.total + chc.total) = 0 THEN 0::double precision
                ELSE chc.total::double precision / (rc.total + chc.total)::double precision
            END AS cache_hit_rate
        FROM request_count rc
        CROSS JOIN cache_hit_count chc
        "#,
    )
    .bind(start)
    .bind(end)
    .fetch_one(pool)
    .await?;

    Ok(stats)
}
