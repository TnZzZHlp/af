use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::types::{ApiType, LbStrategy};

#[derive(Debug, Clone)]
pub struct AliasTargetRow {
    pub id: Uuid,
    pub alias_id: Uuid,
    pub provider_endpoint_id: Uuid,
    pub model_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct AliasTargetDetail {
    pub alias_id: Uuid,
    pub alias_name: String,
    pub alias_strategy: LbStrategy,
    pub alias_target_id: Uuid,
    pub provider_id: Uuid,
    pub provider_name: String,
    pub provider_usage_count: i64,
    pub provider_endpoint_id: Uuid,
    pub endpoint_url: String,
    pub endpoint_timeout_ms: i32,
    pub model_id: Uuid,
    pub model_name: String,
}

pub async fn fetch_alias_targets(
    pool: &PgPool,
    alias_id: Uuid,
) -> anyhow::Result<Vec<AliasTargetRow>> {
    let rows = sqlx::query(
        "SELECT id, alias_id, provider_endpoint_id, model_id
         FROM alias_targets
         WHERE alias_id = $1 AND enabled = true",
    )
    .bind(alias_id)
    .fetch_all(pool)
    .await?;

    let mut targets = Vec::with_capacity(rows.len());
    for row in rows {
        targets.push(AliasTargetRow {
            id: row.try_get("id")?,
            alias_id: row.try_get("alias_id")?,
            provider_endpoint_id: row.try_get("provider_endpoint_id")?,
            model_id: row.try_get("model_id")?,
        });
    }

    Ok(targets)
}

pub async fn fetch_alias_target_details(
    pool: &PgPool,
    alias_name: &str,
    api_type: ApiType,
) -> anyhow::Result<Vec<AliasTargetDetail>> {
    let rows = sqlx::query(
        "SELECT
            a.id AS alias_id,
            a.name AS alias_name,
            a.strategy AS alias_strategy,
            at.id AS alias_target_id,
            p.id AS provider_id,
            p.name AS provider_name,
            p.usage_count AS provider_usage_count,
            pe.id AS provider_endpoint_id,
            pe.url AS endpoint_url,
            pe.timeout_ms AS endpoint_timeout_ms,
            m.id AS model_id,
            m.name AS model_name
         FROM aliases a
         JOIN alias_targets at
           ON at.alias_id = a.id AND at.enabled = true
         JOIN provider_endpoints pe
           ON pe.id = at.provider_endpoint_id AND pe.enabled = true
         JOIN providers p
           ON p.id = pe.provider_id AND p.enabled = true
         JOIN models m
           ON m.id = at.model_id AND m.enabled = true
         WHERE a.name = $1 AND a.api_type = $2 AND a.enabled = true
         ORDER BY p.usage_count DESC",
    )
    .bind(alias_name)
    .bind(api_type)
    .fetch_all(pool)
    .await?;

    let mut details = Vec::with_capacity(rows.len());
    for row in rows {
        details.push(AliasTargetDetail {
            alias_id: row.try_get("alias_id")?,
            alias_name: row.try_get("alias_name")?,
            alias_strategy: row.try_get("alias_strategy")?,
            alias_target_id: row.try_get("alias_target_id")?,
            provider_id: row.try_get("provider_id")?,
            provider_name: row.try_get("provider_name")?,
            provider_usage_count: row.try_get("provider_usage_count")?,
            provider_endpoint_id: row.try_get("provider_endpoint_id")?,
            endpoint_url: row.try_get("endpoint_url")?,
            endpoint_timeout_ms: row.try_get("endpoint_timeout_ms")?,
            model_id: row.try_get("model_id")?,
            model_name: row.try_get("model_name")?,
        });
    }

    Ok(details)
}
