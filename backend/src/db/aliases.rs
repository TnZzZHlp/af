use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alias {
    pub id: Uuid,
    pub name: String,
    pub enabled: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

pub async fn list_aliases(pool: &PgPool, page: i64, page_size: i64) -> AppResult<Vec<Alias>> {
    let offset = (page - 1) * page_size;
    let rows = sqlx::query!(
        "SELECT id, name, enabled, created_at
         FROM aliases
         ORDER BY created_at DESC
         LIMIT $1 OFFSET $2",
        page_size,
        offset
    )
    .fetch_all(pool)
    .await?;

    let mut aliases = Vec::with_capacity(rows.len());
    for row in rows {
        aliases.push(Alias {
            id: row.id,
            name: row.name,
            enabled: row.enabled,
            created_at: row.created_at,
        });
    }

    Ok(aliases)
}

pub async fn get_alias(pool: &PgPool, id: Uuid) -> AppResult<Option<Alias>> {
    let row = sqlx::query!(
        "SELECT id, name, enabled, created_at
         FROM aliases
         WHERE id = $1",
        id
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(Alias {
        id: row.id,
        name: row.name,
        enabled: row.enabled,
        created_at: row.created_at,
    }))
}

pub struct CreateAliasParams {
    pub name: String,
}

pub async fn create_alias(pool: &PgPool, params: CreateAliasParams) -> AppResult<Alias> {
    let row = sqlx::query!(
        "INSERT INTO aliases (name)
         VALUES ($1)
         RETURNING id, name, enabled, created_at",
        params.name
    )
    .fetch_one(pool)
    .await?;

    Ok(Alias {
        id: row.id,
        name: row.name,
        enabled: row.enabled,
        created_at: row.created_at,
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
) -> AppResult<Option<Alias>> {
    let row = sqlx::query!(
        "UPDATE aliases
         SET name = COALESCE($1, name),
             enabled = COALESCE($2, enabled)
         WHERE id = $3
         RETURNING id, name, enabled, created_at",
        params.name,
        params.enabled,
        id
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(Alias {
        id: row.id,
        name: row.name,
        enabled: row.enabled,
        created_at: row.created_at,
    }))
}

pub async fn delete_alias(pool: &PgPool, id: Uuid) -> AppResult<bool> {
    let result = sqlx::query!("DELETE FROM aliases WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
