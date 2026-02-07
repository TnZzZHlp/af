use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayKey {
    pub id: Uuid,
    pub name: Option<String>,
    pub key: String,
    pub enabled: bool,
    pub rate_limit_rps: Option<i32>,
    pub rate_limit_rpm: Option<i32>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

pub async fn fetch_gateway_key(pool: &PgPool, api_key: &str) -> anyhow::Result<Option<GatewayKey>> {
    let row = sqlx::query!(
        "SELECT id, name, key, enabled, rate_limit_rps, rate_limit_rpm, created_at
         FROM gateway_keys
         WHERE key = $1 AND enabled = true
         LIMIT 1",
        api_key
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(GatewayKey {
        id: row.id,
        name: row.name,
        key: row.key,
        enabled: row.enabled,
        rate_limit_rps: row.rate_limit_rps,
        rate_limit_rpm: row.rate_limit_rpm,
        created_at: row.created_at,
    }))
}

pub async fn fetch_gateway_key_by_id(
    pool: &PgPool,
    id: Uuid,
) -> anyhow::Result<Option<GatewayKey>> {
    let row = sqlx::query!(
        "SELECT id, name, key, enabled, rate_limit_rps, rate_limit_rpm, created_at
         FROM gateway_keys
         WHERE id = $1",
        id
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(GatewayKey {
        id: row.id,
        name: row.name,
        key: row.key,
        enabled: row.enabled,
        rate_limit_rps: row.rate_limit_rps,
        rate_limit_rpm: row.rate_limit_rpm,
        created_at: row.created_at,
    }))
}

pub async fn fetch_limits(
    pool: &PgPool,
    gateway_key_id: Uuid,
) -> anyhow::Result<(Option<i32>, Option<i32>)> {
    let row = sqlx::query!(
        "SELECT rate_limit_rps, rate_limit_rpm
         FROM gateway_keys
         WHERE id = $1 AND enabled = true
         LIMIT 1",
        gateway_key_id
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok((None, None));
    };

    Ok((row.rate_limit_rps, row.rate_limit_rpm))
}

pub async fn list_gateway_keys(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> anyhow::Result<Vec<GatewayKey>> {
    let rows = sqlx::query!(
        "SELECT id, name, key, enabled, rate_limit_rps, rate_limit_rpm, created_at
         FROM gateway_keys
         ORDER BY created_at DESC
         LIMIT $1 OFFSET $2",
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    let mut keys = Vec::with_capacity(rows.len());
    for row in rows {
        keys.push(GatewayKey {
            id: row.id,
            name: row.name,
            key: row.key,
            enabled: row.enabled,
            rate_limit_rps: row.rate_limit_rps,
            rate_limit_rpm: row.rate_limit_rpm,
            created_at: row.created_at,
        });
    }

    Ok(keys)
}

pub struct CreateGatewayKeyParams {
    pub name: Option<String>,
    pub key: String,
    pub rate_limit_rps: Option<i32>,
    pub rate_limit_rpm: Option<i32>,
}

pub async fn create_gateway_key(
    pool: &PgPool,
    params: CreateGatewayKeyParams,
) -> anyhow::Result<GatewayKey> {
    let row = sqlx::query!(
        "INSERT INTO gateway_keys (name, key, rate_limit_rps, rate_limit_rpm)
         VALUES ($1, $2, $3, $4)
         RETURNING id, name, key, enabled, rate_limit_rps, rate_limit_rpm, created_at",
        params.name,
        params.key,
        params.rate_limit_rps,
        params.rate_limit_rpm
    )
    .fetch_one(pool)
    .await?;

    Ok(GatewayKey {
        id: row.id,
        name: row.name,
        key: row.key,
        enabled: row.enabled,
        rate_limit_rps: row.rate_limit_rps,
        rate_limit_rpm: row.rate_limit_rpm,
        created_at: row.created_at,
    })
}

pub struct UpdateGatewayKeyParams {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub rate_limit_rps: Option<i32>,
    pub rate_limit_rpm: Option<i32>,
}

pub async fn update_gateway_key(
    pool: &PgPool,
    id: Uuid,
    params: UpdateGatewayKeyParams,
) -> anyhow::Result<Option<GatewayKey>> {
    let row = sqlx::query!(
        "UPDATE gateway_keys
         SET name = COALESCE($1, name),
             enabled = COALESCE($2, enabled),
             rate_limit_rps = $3,
             rate_limit_rpm = $4
         WHERE id = $5
         RETURNING id, name, key, enabled, rate_limit_rps, rate_limit_rpm, created_at",
        params.name,
        params.enabled,
        params.rate_limit_rps,
        params.rate_limit_rpm,
        id
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(GatewayKey {
        id: row.id,
        name: row.name,
        key: row.key,
        enabled: row.enabled,
        rate_limit_rps: row.rate_limit_rps,
        rate_limit_rpm: row.rate_limit_rpm,
        created_at: row.created_at,
    }))
}

pub async fn delete_gateway_key(pool: &PgPool, id: Uuid) -> anyhow::Result<bool> {
    let result = sqlx::query!("DELETE FROM gateway_keys WHERE id = $1", id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}
