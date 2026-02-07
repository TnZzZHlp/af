use crate::db::{
    alias_targets::{
        self, AliasTarget, AliasTargetDetail, CreateAliasTargetParams, UpdateAliasTargetParams,
    },
    aliases::{self, Alias, CreateAliasParams, UpdateAliasParams},
};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list_aliases(pool: &PgPool, page: i64, page_size: i64) -> anyhow::Result<Vec<Alias>> {
    aliases::list_aliases(pool, page, page_size).await
}

pub async fn create_alias(pool: &PgPool, name: String) -> anyhow::Result<Alias> {
    aliases::create_alias(pool, CreateAliasParams { name }).await
}

pub async fn get_alias(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<Alias>> {
    aliases::get_alias(pool, id).await
}

pub async fn update_alias(
    pool: &PgPool,
    id: Uuid,
    name: Option<String>,
    enabled: Option<bool>,
) -> anyhow::Result<Option<Alias>> {
    aliases::update_alias(pool, id, UpdateAliasParams { name, enabled }).await
}

pub async fn delete_alias(pool: &PgPool, id: Uuid) -> anyhow::Result<bool> {
    aliases::delete_alias(pool, id).await
}

// Alias Targets

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
    model_id: String,
) -> anyhow::Result<AliasTarget> {
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
    provider_id: Option<Uuid>,
    model_id: Option<String>,
    enabled: Option<bool>,
) -> anyhow::Result<Option<AliasTarget>> {
    alias_targets::update_alias_target(
        pool,
        id,
        UpdateAliasTargetParams {
            provider_id,
            model_id,
            enabled,
        },
    )
    .await
}

pub async fn delete_alias_target(pool: &PgPool, id: Uuid) -> anyhow::Result<bool> {
    alias_targets::delete_alias_target(pool, id).await
}
