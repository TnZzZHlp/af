use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use super::types::ApiType;
use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderEndpoint {
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
) -> AppResult<Vec<ProviderEndpoint>> {
    let rows = sqlx::query!(
        "SELECT
            id,
            provider_id,
            api_type as \"api_type: ApiType\",
            url,
            enabled,
            created_at
         FROM provider_endpoints
         WHERE provider_id = $1
         ORDER BY created_at DESC",
        provider_id
    )
    .fetch_all(pool)
    .await?;

    let mut endpoints = Vec::with_capacity(rows.len());
    for row in rows {
        endpoints.push(ProviderEndpoint {
            id: row.id,
            provider_id: row.provider_id,
            api_type: row.api_type,
            url: row.url,
            enabled: row.enabled,
            created_at: row.created_at,
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
) -> AppResult<ProviderEndpoint> {
    let row = sqlx::query!(
        "INSERT INTO provider_endpoints (provider_id, api_type, url)
         VALUES ($1, $2::api_type, $3)
         RETURNING id, provider_id, api_type as \"api_type: ApiType\", url, enabled, created_at",
        params.provider_id,
        params.api_type as _,
        params.url
    )
    .fetch_one(pool)
    .await?;

    Ok(ProviderEndpoint {
        id: row.id,
        provider_id: row.provider_id,
        api_type: row.api_type,
        url: row.url,
        enabled: row.enabled,
        created_at: row.created_at,
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
) -> AppResult<Option<ProviderEndpoint>> {
    let row = sqlx::query!(
        "UPDATE provider_endpoints
         SET url = COALESCE($1, url),
             enabled = COALESCE($2, enabled)
         WHERE id = $3
         RETURNING id, provider_id, api_type as \"api_type: ApiType\", url, enabled, created_at",
        params.url,
        params.enabled,
        id
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(ProviderEndpoint {
        id: row.id,
        provider_id: row.provider_id,
        api_type: row.api_type,
        url: row.url,
        enabled: row.enabled,
        created_at: row.created_at,
    }))
}

pub async fn delete_endpoint(pool: &PgPool, id: Uuid) -> AppResult<bool> {
    let result = sqlx::query!("DELETE FROM provider_endpoints WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
