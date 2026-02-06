use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use time::OffsetDateTime;
use uuid::Uuid;

use super::types::ApiType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderEndpointRow {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub api_type: ApiType,
    pub url: String,
    pub usage_count: i64,
    pub timeout_ms: i32,
    pub enabled: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

pub async fn fetch_provider_endpoints(
    pool: &PgPool,
    provider_id: Uuid,
    api_type: ApiType,
) -> anyhow::Result<Vec<ProviderEndpointRow>> {
    let rows = sqlx::query(
        "SELECT
            id,
            provider_id,
            api_type,
            url,
            usage_count,
            timeout_ms,
            enabled,
            created_at
         FROM provider_endpoints
         WHERE provider_id = $1
           AND api_type = $2
           AND enabled = true
         ORDER BY usage_count ASC",
    )
    .bind(provider_id)
    .bind(api_type)
    .fetch_all(pool)
    .await?;

    let mut endpoints = Vec::with_capacity(rows.len());
    for row in rows {
        endpoints.push(ProviderEndpointRow {
            id: row.try_get("id")?,
            provider_id: row.try_get("provider_id")?,
            api_type: row.try_get("api_type")?,
            url: row.try_get("url")?,
            usage_count: row.try_get("usage_count")?,
            timeout_ms: row.try_get("timeout_ms")?,
            enabled: row.try_get("enabled")?,
            created_at: row.try_get("created_at")?,
        });
    }

    Ok(endpoints)
}

pub async fn list_endpoints_by_provider(
    pool: &PgPool,
    provider_id: Uuid,
    api_type: ApiType, // Note: The function signature didn't change in my previous read, but wait.
) -> anyhow::Result<Vec<ProviderEndpointRow>> {
// Wait, I am pasting the whole function.
// The signature in `old_string` (from read_file) was:
// pub async fn list_endpoints_by_provider(
//    pool: &PgPool,
//    provider_id: Uuid,
// )
    let rows = sqlx::query(
        "SELECT
            id,
            provider_id,
            api_type,
            url,
            usage_count,
            timeout_ms,
            enabled,
            created_at
         FROM provider_endpoints
         WHERE provider_id = $1
         ORDER BY created_at DESC",
    )
    .bind(provider_id)
    .fetch_all(pool)
    .await?;

    let mut endpoints = Vec::with_capacity(rows.len());
    for row in rows {
        endpoints.push(ProviderEndpointRow {
            id: row.try_get("id")?,
            provider_id: row.try_get("provider_id")?,
            api_type: row.try_get("api_type")?,
            url: row.try_get("url")?,
            usage_count: row.try_get("usage_count")?,
            timeout_ms: row.try_get("timeout_ms")?,
            enabled: row.try_get("enabled")?,
            created_at: row.try_get("created_at")?,
        });
    }

    Ok(endpoints)
}

pub struct CreateEndpointParams {
    pub provider_id: Uuid,
    pub api_type: ApiType,
    pub url: String,
    pub timeout_ms: Option<i32>,
}

pub async fn create_endpoint(
    pool: &PgPool,
    params: CreateEndpointParams,
) -> anyhow::Result<ProviderEndpointRow> {
    let row = sqlx::query(
        "INSERT INTO provider_endpoints (provider_id, api_type, url, timeout_ms)
         VALUES ($1, $2, $3, COALESCE($4, 60000))
         RETURNING id, provider_id, api_type, url, usage_count, timeout_ms, enabled, created_at",
    )
    .bind(params.provider_id)
    .bind(params.api_type)
    .bind(params.url)
    .bind(params.timeout_ms)
    .fetch_one(pool)
    .await?;

    Ok(ProviderEndpointRow {
        id: row.try_get("id")?,
        provider_id: row.try_get("provider_id")?,
        api_type: row.try_get("api_type")?,
        url: row.try_get("url")?,
        usage_count: row.try_get("usage_count")?,
        timeout_ms: row.try_get("timeout_ms")?,
        enabled: row.try_get("enabled")?,
        created_at: row.try_get("created_at")?,
    })
}

pub struct UpdateEndpointParams {
    pub url: Option<String>,
    pub timeout_ms: Option<i32>,
    pub enabled: Option<bool>,
}

pub async fn update_endpoint(
    pool: &PgPool,
    id: Uuid,
    params: UpdateEndpointParams,
) -> anyhow::Result<Option<ProviderEndpointRow>> {
    let row = sqlx::query(
        "UPDATE provider_endpoints
         SET url = COALESCE($1, url),
             timeout_ms = COALESCE($2, timeout_ms),
             enabled = COALESCE($3, enabled)
         WHERE id = $4
         RETURNING id, provider_id, api_type, url, usage_count, timeout_ms, enabled, created_at",
    )
    .bind(params.url)
    .bind(params.timeout_ms)
    .bind(params.enabled)
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(ProviderEndpointRow {
        id: row.try_get("id")?,
        provider_id: row.try_get("provider_id")?,
        api_type: row.try_get("api_type")?,
        url: row.try_get("url")?,
        usage_count: row.try_get("usage_count")?,
        timeout_ms: row.try_get("timeout_ms")?,
        enabled: row.try_get("enabled")?,
        created_at: row.try_get("created_at")?,
    }))
}

pub async fn delete_endpoint(pool: &PgPool, id: Uuid) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM provider_endpoints WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn increment_usage_count(pool: &PgPool, id: Uuid) -> anyhow::Result<()> {
    sqlx::query("UPDATE provider_endpoints SET usage_count = usage_count + 1 WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
