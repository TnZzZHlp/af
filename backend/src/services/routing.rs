use sqlx::PgPool;
use uuid::Uuid;

use crate::db::{alias_targets, aliases, provider_keys};

#[derive(Debug, Clone)]
pub struct AliasRow {
    pub id: Uuid,
    pub name: String,
    pub api_type: String,
    pub strategy: String,
}

#[derive(Debug, Clone)]
pub struct AliasTargetRow {
    pub id: Uuid,
    pub alias_id: Uuid,
    pub provider_endpoint_id: Uuid,
    pub model_id: Uuid,
    pub weight: i32,
    pub priority: i32,
}

#[derive(Debug, Clone)]
pub struct AliasTargetDetail {
    pub alias_id: Uuid,
    pub alias_name: String,
    pub alias_strategy: String,
    pub alias_target_id: Uuid,
    pub target_weight: i32,
    pub target_priority: i32,
    pub provider_id: Uuid,
    pub provider_name: String,
    pub provider_endpoint_id: Uuid,
    pub endpoint_base_url: String,
    pub endpoint_path: String,
    pub endpoint_timeout_ms: i32,
    pub endpoint_weight: i32,
    pub endpoint_priority: i32,
    pub model_id: Uuid,
    pub model_name: String,
}

#[derive(Debug, Clone)]
pub struct ProviderKeyRow {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub name: Option<String>,
    pub key: String,
    pub weight: i32,
    pub fail_count: i32,
    pub circuit_open_until: Option<sqlx::types::time::OffsetDateTime>,
    pub last_fail_at: Option<sqlx::types::time::OffsetDateTime>,
}

pub async fn fetch_alias(
    pool: &PgPool,
    name: &str,
    api_type: &str,
) -> anyhow::Result<Option<AliasRow>> {
    let row = match aliases::fetch_alias(pool, name, api_type).await? {
        Some(row) => row,
        None => return Ok(None),
    };

    Ok(Some(AliasRow {
        id: row.id,
        name: row.name,
        api_type: row.api_type,
        strategy: row.strategy,
    }))
}

pub async fn fetch_alias_targets(
    pool: &PgPool,
    alias_id: Uuid,
) -> anyhow::Result<Vec<AliasTargetRow>> {
    let rows = alias_targets::fetch_alias_targets(pool, alias_id).await?;
    let mut targets = Vec::with_capacity(rows.len());
    for row in rows {
        targets.push(AliasTargetRow {
            id: row.id,
            alias_id: row.alias_id,
            provider_endpoint_id: row.provider_endpoint_id,
            model_id: row.model_id,
            weight: row.weight,
            priority: row.priority,
        });
    }

    Ok(targets)
}

pub async fn fetch_alias_target_details(
    pool: &PgPool,
    alias_name: &str,
    api_type: &str,
) -> anyhow::Result<Vec<AliasTargetDetail>> {
    let rows = alias_targets::fetch_alias_target_details(pool, alias_name, api_type).await?;
    let mut details = Vec::with_capacity(rows.len());
    for row in rows {
        details.push(AliasTargetDetail {
            alias_id: row.alias_id,
            alias_name: row.alias_name,
            alias_strategy: row.alias_strategy,
            alias_target_id: row.alias_target_id,
            target_weight: row.target_weight,
            target_priority: row.target_priority,
            provider_id: row.provider_id,
            provider_name: row.provider_name,
            provider_endpoint_id: row.provider_endpoint_id,
            endpoint_base_url: row.endpoint_base_url,
            endpoint_path: row.endpoint_path,
            endpoint_timeout_ms: row.endpoint_timeout_ms,
            endpoint_weight: row.endpoint_weight,
            endpoint_priority: row.endpoint_priority,
            model_id: row.model_id,
            model_name: row.model_name,
        });
    }

    Ok(details)
}

pub async fn fetch_provider_keys(
    pool: &PgPool,
    provider_id: Uuid,
) -> anyhow::Result<Vec<ProviderKeyRow>> {
    let rows = provider_keys::fetch_provider_keys(pool, provider_id).await?;
    let mut keys = Vec::with_capacity(rows.len());
    for row in rows {
        keys.push(ProviderKeyRow {
            id: row.id,
            provider_id: row.provider_id,
            name: row.name,
            key: row.key,
            weight: row.weight,
            fail_count: row.fail_count,
            circuit_open_until: row.circuit_open_until,
            last_fail_at: row.last_fail_at,
        });
    }

    Ok(keys)
}
