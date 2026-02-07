use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub usage_count: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

pub async fn fetch_provider_by_id(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<ProviderRow>> {
    let row = sqlx::query!(
        "SELECT id, name, description, enabled, usage_count, created_at
         FROM providers
         WHERE id = $1",
        id
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(ProviderRow {
        id: row.id,
        name: row.name,
        description: row.description,
        enabled: row.enabled,
        usage_count: row.usage_count,
        created_at: row.created_at,
    }))
}

pub async fn list_providers(
    pool: &PgPool,
    page: i64,
    page_size: i64,
) -> anyhow::Result<Vec<ProviderRow>> {
    let offset = (page - 1) * page_size;
    let rows = sqlx::query!(
        "SELECT id, name, description, enabled, usage_count, created_at
         FROM providers
         ORDER BY created_at DESC
         LIMIT $1 OFFSET $2",
        page_size,
        offset
    )
    .fetch_all(pool)
    .await?;

    let mut providers = Vec::with_capacity(rows.len());
    for row in rows {
        providers.push(ProviderRow {
            id: row.id,
            name: row.name,
            description: row.description,
            enabled: row.enabled,
            usage_count: row.usage_count,
            created_at: row.created_at,
        });
    }

    Ok(providers)
}

pub struct CreateProviderParams {
    pub name: String,
    pub description: Option<String>,
}

pub async fn create_provider(
    pool: &PgPool,
    params: CreateProviderParams,
) -> anyhow::Result<ProviderRow> {
    let row = sqlx::query!(
        "INSERT INTO providers (name, description)
         VALUES ($1, $2)
         RETURNING id, name, description, enabled, usage_count, created_at",
        params.name,
        params.description
    )
    .fetch_one(pool)
    .await?;

    Ok(ProviderRow {
        id: row.id,
        name: row.name,
        description: row.description,
        enabled: row.enabled,
        usage_count: row.usage_count,
        created_at: row.created_at,
    })
}

pub struct UpdateProviderParams {
    pub name: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
}

pub async fn update_provider(
    pool: &PgPool,
    id: Uuid,
    params: UpdateProviderParams,
) -> anyhow::Result<Option<ProviderRow>> {
    let row = sqlx::query!(
        "UPDATE providers
         SET name = COALESCE($1, name),
             description = COALESCE($2, description),
             enabled = COALESCE($3, enabled)
         WHERE id = $4
         RETURNING id, name, description, enabled, usage_count, created_at",
        params.name,
        params.description,
        params.enabled,
        id
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(ProviderRow {
        id: row.id,
        name: row.name,
        description: row.description,
        enabled: row.enabled,
        usage_count: row.usage_count,
        created_at: row.created_at,
    }))
}

pub async fn delete_provider(pool: &PgPool, id: Uuid) -> anyhow::Result<bool> {
    let result = sqlx::query!("DELETE FROM providers WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn increment_usage_count(pool: &PgPool, id: Uuid) -> anyhow::Result<()> {
    sqlx::query!(
        "UPDATE providers SET usage_count = usage_count + 1 WHERE id = $1",
        id
    )
    .execute(pool)
    .await?;
    Ok(())
}
