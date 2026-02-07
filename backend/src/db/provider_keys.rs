use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderKey {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub name: Option<String>,
    pub key: String,
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
) -> anyhow::Result<Vec<ProviderKey>> {
    let rows = sqlx::query!(
        "SELECT
            id,
            provider_id,
            name,
            key,
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
        provider_id
    )
    .fetch_all(pool)
    .await?;

    let mut keys = Vec::with_capacity(rows.len());
    for row in rows {
        keys.push(ProviderKey {
            id: row.id,
            provider_id: row.provider_id,
            name: row.name,
            key: row.key,
            usage_count: row.usage_count,
            enabled: row.enabled,
            fail_count: row.fail_count,
            circuit_open_until: row.circuit_open_until,
            last_fail_at: row.last_fail_at,
            created_at: row.created_at,
        });
    }

    Ok(keys)
}

pub async fn list_keys_by_provider(
    pool: &PgPool,
    provider_id: Uuid,
) -> anyhow::Result<Vec<ProviderKey>> {
    let rows = sqlx::query!(
        "SELECT
            id,
            provider_id,
            name,
            key,
            usage_count,
            enabled,
            fail_count,
            circuit_open_until,
            last_fail_at,
            created_at
         FROM provider_keys
         WHERE provider_id = $1
         ORDER BY created_at DESC",
        provider_id
    )
    .fetch_all(pool)
    .await?;

    let mut keys = Vec::with_capacity(rows.len());
    for row in rows {
        keys.push(ProviderKey {
            id: row.id,
            provider_id: row.provider_id,
            name: row.name,
            key: row.key,
            usage_count: row.usage_count,
            enabled: row.enabled,
            fail_count: row.fail_count,
            circuit_open_until: row.circuit_open_until,
            last_fail_at: row.last_fail_at,
            created_at: row.created_at,
        });
    }

    Ok(keys)
}

pub struct CreateKeyParams {
    pub provider_id: Uuid,
    pub name: Option<String>,
    pub key: String,
}

pub async fn create_key(pool: &PgPool, params: CreateKeyParams) -> anyhow::Result<ProviderKey> {
    let row = sqlx::query!(
        "INSERT INTO provider_keys (provider_id, name, key)
         VALUES ($1, $2, $3)
         RETURNING id, provider_id, name, key, usage_count, enabled, fail_count, circuit_open_until, last_fail_at, created_at",
        params.provider_id,
        params.name,
        params.key
    )
    .fetch_one(pool)
    .await?;

    Ok(ProviderKey {
        id: row.id,
        provider_id: row.provider_id,
        name: row.name,
        key: row.key,
        usage_count: row.usage_count,
        enabled: row.enabled,
        fail_count: row.fail_count,
        circuit_open_until: row.circuit_open_until,
        last_fail_at: row.last_fail_at,
        created_at: row.created_at,
    })
}

pub struct UpdateKeyParams {
    pub name: Option<String>,
    pub enabled: Option<bool>,
}

pub async fn update_key(
    pool: &PgPool,
    id: Uuid,
    params: UpdateKeyParams,
) -> anyhow::Result<Option<ProviderKey>> {
    let row = sqlx::query!(
        "UPDATE provider_keys
         SET name = COALESCE($1, name),
             enabled = COALESCE($2, enabled)
         WHERE id = $3
         RETURNING id, provider_id, name, key, usage_count, enabled, fail_count, circuit_open_until, last_fail_at, created_at",
        params.name,
        params.enabled,
        id
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(ProviderKey {
        id: row.id,
        provider_id: row.provider_id,
        name: row.name,
        key: row.key,
        usage_count: row.usage_count,
        enabled: row.enabled,
        fail_count: row.fail_count,
        circuit_open_until: row.circuit_open_until,
        last_fail_at: row.last_fail_at,
        created_at: row.created_at,
    }))
}

pub async fn delete_key(pool: &PgPool, id: Uuid) -> anyhow::Result<bool> {
    let result = sqlx::query!("DELETE FROM provider_keys WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn increment_usage_count(pool: &PgPool, id: Uuid) -> anyhow::Result<()> {
    sqlx::query!(
        "UPDATE provider_keys SET usage_count = usage_count + 1 WHERE id = $1",
        id
    )
    .execute(pool)
    .await?;
    Ok(())
}
