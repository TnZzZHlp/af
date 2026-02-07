use sqlx::PgPool;
use uuid::Uuid;

use crate::db::types::ApiType;
use crate::db::{alias_targets, provider_keys};

#[derive(Debug, Clone)]
pub struct AliasTargetDetail {
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
}

#[derive(Debug, Clone)]
pub struct ProviderKeyRow {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub name: Option<String>,
    pub key: String,
    pub usage_count: i64,
    pub fail_count: i32,
    pub circuit_open_until: Option<sqlx::types::time::OffsetDateTime>,
    pub last_fail_at: Option<sqlx::types::time::OffsetDateTime>,
}

pub async fn fetch_alias_target_details(
    pool: &PgPool,
    alias_name: &str,
    api_type: ApiType,
) -> anyhow::Result<Vec<AliasTargetDetail>> {
    let rows = alias_targets::fetch_alias_target_details(pool, alias_name, api_type).await?;
    let mut details = Vec::with_capacity(rows.len());
    for row in rows {
        details.push(AliasTargetDetail {
            alias_id: row.alias_id,
            alias_name: row.alias_name,
            alias_target_id: row.alias_target_id,
            provider_id: row.provider_id,
            provider_name: row.provider_name,
            provider_usage_count: row.provider_usage_count,
            provider_endpoint_id: row.provider_endpoint_id,
            endpoint_url: row.endpoint_url,
            model_id: row.model_id,
            enabled: row.enabled,
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
            usage_count: row.usage_count,
            fail_count: row.fail_count,
            circuit_open_until: row.circuit_open_until,
            last_fail_at: row.last_fail_at,
        });
    }

    Ok(keys)
}
