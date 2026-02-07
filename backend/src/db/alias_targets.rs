use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use super::types::ApiType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasTargetRow {
    pub id: Uuid,
    pub alias_id: Uuid,
    pub provider_id: Uuid,
    pub model_id: String,
    pub enabled: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasTargetDetail {
    pub id: Uuid,
    pub alias_id: Uuid,
    pub alias_name: String,
    pub alias_target_id: Uuid,
    pub provider_id: Uuid,
    pub provider_name: String,
    pub provider_usage_count: i64,
    pub provider_endpoint_id: Option<Uuid>,
    pub endpoint_url: Option<String>,
    pub model_id: String,
    pub enabled: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

pub async fn fetch_all_alias_target_details(
    pool: &PgPool,
    alias_id: Uuid,
) -> anyhow::Result<Vec<AliasTargetDetail>> {
    let rows = sqlx::query!(
        "SELECT
            at.id,
            a.id AS alias_id,
            a.name AS alias_name,
            at.id AS alias_target_id,
            p.id AS provider_id,
            p.name AS provider_name,
            p.usage_count AS provider_usage_count,
            at.model_id,
            at.enabled,
            at.created_at
         FROM aliases a
         JOIN alias_targets at
           ON at.alias_id = a.id
         JOIN providers p
           ON p.id = at.provider_id
         WHERE a.id = $1
         ORDER BY p.usage_count ASC, p.id",
        alias_id
    )
    .fetch_all(pool)
    .await?;

    let mut details = Vec::with_capacity(rows.len());
    for row in rows {
        details.push(AliasTargetDetail {
            id: row.id,
            alias_id: row.alias_id,
            alias_name: row.alias_name,
            alias_target_id: row.alias_target_id,
            provider_id: row.provider_id,
            provider_name: row.provider_name,
            provider_usage_count: row.provider_usage_count,
            provider_endpoint_id: None,
            endpoint_url: None,
            model_id: row.model_id,
            enabled: row.enabled,
            created_at: row.created_at,
        });
    }

    Ok(details)
}

pub struct CreateAliasTargetParams {
    pub alias_id: Uuid,
    pub provider_id: Uuid,
    pub model_id: String,
}

pub async fn create_alias_target(
    pool: &PgPool,
    params: CreateAliasTargetParams,
) -> anyhow::Result<AliasTargetRow> {
    let row = sqlx::query!(
        "INSERT INTO alias_targets (alias_id, provider_id, model_id)
         VALUES ($1, $2, $3)
         RETURNING id, alias_id, provider_id, model_id, enabled, created_at",
        params.alias_id,
        params.provider_id,
        params.model_id
    )
    .fetch_one(pool)
    .await?;

    Ok(AliasTargetRow {
        id: row.id,
        alias_id: row.alias_id,
        provider_id: row.provider_id,
        model_id: row.model_id,
        enabled: row.enabled,
        created_at: row.created_at,
    })
}

pub struct UpdateAliasTargetParams {
    pub provider_id: Option<Uuid>,
    pub model_id: Option<String>,
    pub enabled: Option<bool>,
}

pub async fn update_alias_target(
    pool: &PgPool,
    id: Uuid,
    params: UpdateAliasTargetParams,
) -> anyhow::Result<Option<AliasTargetRow>> {
    let row = sqlx::query!(
        "UPDATE alias_targets
         SET
            provider_id = COALESCE($1, provider_id),
            model_id = COALESCE($2, model_id),
            enabled = COALESCE($3, enabled)
         WHERE id = $4
         RETURNING id, alias_id, provider_id, model_id, enabled, created_at",
        params.provider_id,
        params.model_id,
        params.enabled,
        id
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(AliasTargetRow {
        id: row.id,
        alias_id: row.alias_id,
        provider_id: row.provider_id,
        model_id: row.model_id,
        enabled: row.enabled,
        created_at: row.created_at,
    }))
}

pub async fn delete_alias_target(pool: &PgPool, id: Uuid) -> anyhow::Result<bool> {
    let result = sqlx::query!("DELETE FROM alias_targets WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn fetch_alias_target_details(
    pool: &PgPool,
    alias_name: &str,
    api_type: ApiType,
) -> anyhow::Result<Vec<AliasTargetDetail>> {
    // We join providers directly.
    // We join provider_endpoints on provider_id and api_type to find a valid endpoint.
    // We distinct on provider_id to avoid duplicates if multiple endpoints exist.
    // We just pick one endpoint (e.g. max ID or whatever) since we assume any valid endpoint for the provider+api_type works.
    let rows = sqlx::query!(
        "SELECT DISTINCT ON (p.usage_count, p.id)
            at.id,
            a.id AS alias_id,
            a.name AS alias_name,
            at.id AS alias_target_id,
            p.id AS provider_id,
            p.name AS provider_name,
            p.usage_count AS provider_usage_count,
            pe.id AS provider_endpoint_id,
            pe.url AS endpoint_url,
            at.model_id,
            at.enabled,
            at.created_at
         FROM aliases a
         JOIN alias_targets at
           ON at.alias_id = a.id AND at.enabled = true
         JOIN providers p
           ON p.id = at.provider_id AND p.enabled = true
         JOIN provider_endpoints pe
           ON pe.provider_id = p.id AND pe.api_type = $2 AND pe.enabled = true
         WHERE a.name = $1 AND a.enabled = true
         ORDER BY p.usage_count ASC, p.id",
        alias_name,
        api_type as _
    )
    .fetch_all(pool)
    .await?;

    let mut details = Vec::with_capacity(rows.len());
    for row in rows {
        details.push(AliasTargetDetail {
            id: row.id,
            alias_id: row.alias_id,
            alias_name: row.alias_name,
            alias_target_id: row.alias_target_id,
            provider_id: row.provider_id,
            provider_name: row.provider_name,
            provider_usage_count: row.provider_usage_count,
            provider_endpoint_id: Some(row.provider_endpoint_id),
            endpoint_url: Some(row.endpoint_url),
            model_id: row.model_id,
            enabled: row.enabled,
            created_at: row.created_at,
        });
    }

    Ok(details)
}
