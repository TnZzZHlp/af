use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderKeyRow {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub name: Option<String>,
    pub key: String,
    pub weight: i32,
    pub usage_count: i64,
    pub enabled: bool,
    pub fail_count: i32,
    #[serde(with = "time::serde::rfc3339::option")]
    pub circuit_open_until: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_fail_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

pub async fn fetch_provider_keys(
    pool: &PgPool,
    provider_id: Uuid,
) -> anyhow::Result<Vec<ProviderKeyRow>> {
    let rows = sqlx::query(
        "SELECT
            id,
            provider_id,
            name,
            key,
            weight,
            usage_count,
            enabled,
            fail_count,
            circuit_open_until,
            last_fail_at,
            created_at
         FROM provider_keys
         WHERE provider_id = $1
           AND enabled = true
           AND (circuit_open_until IS NULL OR circuit_open_until <= now())
         ORDER BY usage_count ASC",
    )
    .bind(provider_id)
    .fetch_all(pool)
    .await?;

    let mut keys = Vec::with_capacity(rows.len());
    for row in rows {
        keys.push(ProviderKeyRow {
            id: row.try_get("id")?,
            provider_id: row.try_get("provider_id")?,
            name: row.try_get("name")?,
            key: row.try_get("key")?,
            weight: row.try_get("weight")?,
            usage_count: row.try_get("usage_count")?,
            enabled: row.try_get("enabled")?,
            fail_count: row.try_get("fail_count")?,
            circuit_open_until: row.try_get("circuit_open_until")?,
            last_fail_at: row.try_get("last_fail_at")?,
            created_at: row.try_get("created_at")?,
        });
    }

    Ok(keys)
}

pub async fn list_keys_by_provider(
    pool: &PgPool,
    provider_id: Uuid,
) -> anyhow::Result<Vec<ProviderKeyRow>> {
    let rows = sqlx::query(
        "SELECT
            id,
            provider_id,
            name,
            key,
            weight,
            usage_count,
            enabled,
            fail_count,
            circuit_open_until,
            last_fail_at,
            created_at
         FROM provider_keys
         WHERE provider_id = $1
         ORDER BY created_at DESC",
    )
    .bind(provider_id)
    .fetch_all(pool)
    .await?;

    let mut keys = Vec::with_capacity(rows.len());
    for row in rows {
        keys.push(ProviderKeyRow {
            id: row.try_get("id")?,
            provider_id: row.try_get("provider_id")?,
            name: row.try_get("name")?,
            key: row.try_get("key")?,
            weight: row.try_get("weight")?,
            usage_count: row.try_get("usage_count")?,
            enabled: row.try_get("enabled")?,
            fail_count: row.try_get("fail_count")?,
            circuit_open_until: row.try_get("circuit_open_until")?,
            last_fail_at: row.try_get("last_fail_at")?,
            created_at: row.try_get("created_at")?,
        });
    }

    Ok(keys)
}

pub struct CreateKeyParams {
    pub provider_id: Uuid,
    pub name: Option<String>,
    pub key: String,
    pub weight: Option<i32>,
}

pub async fn create_key(pool: &PgPool, params: CreateKeyParams) -> anyhow::Result<ProviderKeyRow> {
    let row = sqlx::query(
        "INSERT INTO provider_keys (provider_id, name, key, weight)
         VALUES ($1, $2, $3, COALESCE($4, 1))
         RETURNING id, provider_id, name, key, weight, usage_count, enabled, fail_count, circuit_open_until, last_fail_at, created_at",
    )
    .bind(params.provider_id)
    .bind(params.name)
    .bind(params.key)
    .bind(params.weight)
    .fetch_one(pool)
    .await?;

    Ok(ProviderKeyRow {
        id: row.try_get("id")?,
        provider_id: row.try_get("provider_id")?,
        name: row.try_get("name")?,
        key: row.try_get("key")?,
        weight: row.try_get("weight")?,
        usage_count: row.try_get("usage_count")?,
        enabled: row.try_get("enabled")?,
        fail_count: row.try_get("fail_count")?,
        circuit_open_until: row.try_get("circuit_open_until")?,
        last_fail_at: row.try_get("last_fail_at")?,
        created_at: row.try_get("created_at")?,
    })
}

pub struct UpdateKeyParams {
    pub name: Option<String>,
    pub weight: Option<i32>,
    pub enabled: Option<bool>,
}

pub async fn update_key(
    pool: &PgPool,
    id: Uuid,
    params: UpdateKeyParams,
) -> anyhow::Result<Option<ProviderKeyRow>> {
    let row = sqlx::query(
        "UPDATE provider_keys
         SET name = COALESCE($1, name),
             weight = COALESCE($2, weight),
             enabled = COALESCE($3, enabled)
         WHERE id = $4
         RETURNING id, provider_id, name, key, weight, usage_count, enabled, fail_count, circuit_open_until, last_fail_at, created_at",
    )
    .bind(params.name)
    .bind(params.weight)
    .bind(params.enabled)
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(ProviderKeyRow {
        id: row.try_get("id")?,
        provider_id: row.try_get("provider_id")?,
        name: row.try_get("name")?,
        key: row.try_get("key")?,
        weight: row.try_get("weight")?,
        usage_count: row.try_get("usage_count")?,
        enabled: row.try_get("enabled")?,
        fail_count: row.try_get("fail_count")?,
        circuit_open_until: row.try_get("circuit_open_until")?,
        last_fail_at: row.try_get("last_fail_at")?,
        created_at: row.try_get("created_at")?,
    }))
}

pub async fn delete_key(pool: &PgPool, id: Uuid) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM provider_keys WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn increment_usage_count(pool: &PgPool, id: Uuid) -> anyhow::Result<()> {
    sqlx::query("UPDATE provider_keys SET usage_count = usage_count + 1 WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
