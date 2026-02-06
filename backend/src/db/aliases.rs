use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasRow {
    pub id: Uuid,
    pub name: String,
    pub enabled: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

pub async fn list_aliases(
    pool: &PgPool,
    page: i64,
    page_size: i64,
) -> anyhow::Result<Vec<AliasRow>> {
    let offset = (page - 1) * page_size;
    let rows = sqlx::query(
        "SELECT id, name, enabled, created_at
         FROM aliases
         ORDER BY created_at DESC
         LIMIT $1 OFFSET $2",
    )
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let mut aliases = Vec::with_capacity(rows.len());
    for row in rows {
        aliases.push(AliasRow {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            enabled: row.try_get("enabled")?,
            created_at: row.try_get("created_at")?,
        });
    }

    Ok(aliases)
}

pub async fn fetch_alias(
    pool: &PgPool,
    name: &str,
) -> anyhow::Result<Option<AliasRow>> {
    let row = sqlx::query(
        "SELECT id, name, enabled, created_at
         FROM aliases
         WHERE name = $1 AND enabled = true
         LIMIT 1",
    )
    .bind(name)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(AliasRow {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        enabled: row.try_get("enabled")?,
        created_at: row.try_get("created_at")?,
    }))
}

pub async fn get_alias(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<AliasRow>> {
    let row = sqlx::query(
        "SELECT id, name, enabled, created_at
         FROM aliases
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(AliasRow {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        enabled: row.try_get("enabled")?,
        created_at: row.try_get("created_at")?,
    }))
}

pub struct CreateAliasParams {
    pub name: String,
}

pub async fn create_alias(
    pool: &PgPool,
    params: CreateAliasParams,
) -> anyhow::Result<AliasRow> {
    let row = sqlx::query(
        "INSERT INTO aliases (name)
         VALUES ($1)
         RETURNING id, name, enabled, created_at",
    )
    .bind(params.name)
    .fetch_one(pool)
    .await?;

    Ok(AliasRow {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        enabled: row.try_get("enabled")?,
        created_at: row.try_get("created_at")?,
    })
}

pub struct UpdateAliasParams {
    pub name: Option<String>,
    pub enabled: Option<bool>,
}

pub async fn update_alias(
    pool: &PgPool,
    id: Uuid,
    params: UpdateAliasParams,
) -> anyhow::Result<Option<AliasRow>> {
    let row = sqlx::query(
        "UPDATE aliases
         SET name = COALESCE($1, name),
             enabled = COALESCE($2, enabled)
         WHERE id = $3
         RETURNING id, name, enabled, created_at",
    )
    .bind(params.name)
    .bind(params.enabled)
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(AliasRow {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        enabled: row.try_get("enabled")?,
        created_at: row.try_get("created_at")?,
    }))
}

pub async fn delete_alias(pool: &PgPool, id: Uuid) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM aliases WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
