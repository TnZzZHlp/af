use sqlx::PgPool;
use uuid::Uuid;

use crate::db::alias_targets::AliasTargetDetail;
use crate::db::provider_keys::ProviderKey;
use crate::db::types::ApiType;
use crate::db::{alias_targets, provider_keys};

pub async fn fetch_alias_target_details(
    pool: &PgPool,
    alias_name: &str,
    api_type: ApiType,
) -> anyhow::Result<Vec<AliasTargetDetail>> {
    let rows = alias_targets::fetch_alias_target_details(pool, alias_name, api_type).await?;

    Ok(rows)
}

pub async fn fetch_provider_keys(
    pool: &PgPool,
    provider_id: Uuid,
) -> anyhow::Result<Vec<ProviderKey>> {
    let rows = provider_keys::fetch_provider_keys(pool, provider_id).await?;

    Ok(rows)
}
