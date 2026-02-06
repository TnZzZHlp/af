use crate::db::types::ApiType;
use crate::db::{
    alias_targets::{
        self, AliasTargetDetail, AliasTargetRow, CreateAliasTargetParams, UpdateAliasTargetParams,
    },
    aliases::{self, AliasRow, CreateAliasParams, UpdateAliasParams},
};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list_aliases(
    pool: &PgPool,
    page: i64,
    page_size: i64,
) -> anyhow::Result<Vec<AliasRow>> {
    aliases::list_aliases(pool, page, page_size).await
}

pub async fn create_alias(
    pool: &PgPool,
    name: String,
) -> anyhow::Result<AliasRow> {
    aliases::create_alias(pool, CreateAliasParams { name }).await
}

pub async fn get_alias(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<AliasRow>> {
    aliases::get_alias(pool, id).await
}

pub async fn update_alias(
    pool: &PgPool,
    id: Uuid,
    name: Option<String>,
    enabled: Option<bool>,
) -> anyhow::Result<Option<AliasRow>> {
    aliases::update_alias(pool, id, UpdateAliasParams { name, enabled }).await
}

pub async fn delete_alias(pool: &PgPool, id: Uuid) -> anyhow::Result<bool> {
    aliases::delete_alias(pool, id).await
}

// Alias Targets

pub async fn fetch_alias_targets(
    pool: &PgPool,
    alias_id: Uuid,
) -> anyhow::Result<Vec<AliasTargetRow>> {
    alias_targets::fetch_alias_targets(pool, alias_id).await
}

pub async fn fetch_alias_target_details(
    pool: &PgPool,
    alias_name: &str,
    api_type: ApiType,
) -> anyhow::Result<Vec<AliasTargetDetail>> {
    alias_targets::fetch_alias_target_details(pool, alias_name, api_type).await
}

pub async fn fetch_all_alias_target_details(
    pool: &PgPool,
    alias_id: Uuid,
) -> anyhow::Result<Vec<AliasTargetDetail>> {
    alias_targets::fetch_all_alias_target_details(pool, alias_id).await
}

pub async fn create_alias_target(
    pool: &PgPool,
    alias_id: Uuid,
    provider_id: Uuid,
    model_id: Uuid,
) -> anyhow::Result<AliasTargetRow> {
    alias_targets::create_alias_target(
        pool,
        CreateAliasTargetParams {
            alias_id,
            provider_id,
            model_id,
        },
    )
    .await
}

pub async fn update_alias_target(
    pool: &PgPool,
    id: Uuid,
    enabled: Option<bool>,
) -> anyhow::Result<Option<AliasTargetRow>> {
    alias_targets::update_alias_target(pool, id, UpdateAliasTargetParams { enabled }).await
}

pub async fn delete_alias_target(pool: &PgPool, id: Uuid) -> anyhow::Result<bool> {
    alias_targets::delete_alias_target(pool, id).await
}
