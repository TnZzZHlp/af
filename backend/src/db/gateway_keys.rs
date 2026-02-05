use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct GatewayKey {
    pub id: Uuid,
    pub name: Option<String>,
    pub rate_limit_rps: Option<i32>,
    pub rate_limit_rpm: Option<i32>,
}

pub async fn fetch_gateway_key(pool: &PgPool, api_key: &str) -> anyhow::Result<Option<GatewayKey>> {
    let row = sqlx::query(
        "SELECT id, name, rate_limit_rps, rate_limit_rpm\n         FROM gateway_keys\n         WHERE key = $1 AND enabled = true\n         LIMIT 1",
    )
    .bind(api_key)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(GatewayKey {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        rate_limit_rps: row.try_get("rate_limit_rps")?,
        rate_limit_rpm: row.try_get("rate_limit_rpm")?,
    }))
}

pub async fn fetch_limits(
    pool: &PgPool,
    gateway_key_id: Uuid,
) -> anyhow::Result<(Option<i32>, Option<i32>)> {
    let row = sqlx::query(
        "SELECT rate_limit_rps, rate_limit_rpm\n         FROM gateway_keys\n         WHERE id = $1 AND enabled = true\n         LIMIT 1",
    )
    .bind(gateway_key_id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok((None, None));
    };

    Ok((
        row.try_get("rate_limit_rps")?,
        row.try_get("rate_limit_rpm")?,
    ))
}
