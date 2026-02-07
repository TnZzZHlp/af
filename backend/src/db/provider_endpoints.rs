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
    pub enabled: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

pub async fn list_endpoints_by_provider(
    pool: &PgPool,
    provider_id: Uuid,
) -> anyhow::Result<Vec<ProviderEndpointRow>> {
    let rows = sqlx::query(
        "SELECT
            id,
            provider_id,
            api_type,
            url,
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
}

pub async fn create_endpoint(
    pool: &PgPool,
    params: CreateEndpointParams,
) -> anyhow::Result<ProviderEndpointRow> {
    let row = sqlx::query(
        "INSERT INTO provider_endpoints (provider_id, api_type, url)
         VALUES ($1, $2, $3)
         RETURNING id, provider_id, api_type, url, enabled, created_at",
    )
    .bind(params.provider_id)
    .bind(params.api_type)
    .bind(params.url)
    .fetch_one(pool)
    .await?;

    Ok(ProviderEndpointRow {
        id: row.try_get("id")?,
        provider_id: row.try_get("provider_id")?,
        api_type: row.try_get("api_type")?,
        url: row.try_get("url")?,
        enabled: row.try_get("enabled")?,
        created_at: row.try_get("created_at")?,
    })
}

pub struct UpdateEndpointParams {
    pub url: Option<String>,
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
             enabled = COALESCE($2, enabled)
         WHERE id = $3
         RETURNING id, provider_id, api_type, url, enabled, created_at",
    )
    .bind(params.url)
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