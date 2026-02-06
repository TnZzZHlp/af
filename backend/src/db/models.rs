use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use time::OffsetDateTime;
use uuid::Uuid;

use super::types::ApiType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRow {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub api_type: ApiType,
    pub name: String,
    pub enabled: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

pub async fn list_models(
    pool: &PgPool,
    provider_id: Uuid,
) -> anyhow::Result<Vec<ModelRow>> {
    let rows = sqlx::query(
        "SELECT id, provider_id, api_type, name, enabled, created_at
         FROM models
         WHERE provider_id = $1
         ORDER BY created_at DESC",
    )
    .bind(provider_id)
    .fetch_all(pool)
    .await?;

    let mut models = Vec::with_capacity(rows.len());
    for row in rows {
        models.push(ModelRow {
            id: row.try_get("id")?,
            provider_id: row.try_get("provider_id")?,
            api_type: row.try_get("api_type")?,
            name: row.try_get("name")?,
            enabled: row.try_get("enabled")?,
            created_at: row.try_get("created_at")?,
        });
    }

    Ok(models)
}

pub async fn fetch_models(
    pool: &PgPool,
    provider_id: Uuid,
    api_type: ApiType,
) -> anyhow::Result<Vec<ModelRow>> {
    let rows = sqlx::query(
        "SELECT id, provider_id, api_type, name, enabled, created_at
         FROM models
         WHERE provider_id = $1
           AND api_type = $2
           AND enabled = true
         ORDER BY name ASC",
    )
    .bind(provider_id)
    .bind(api_type)
    .fetch_all(pool)
    .await?;

    let mut models = Vec::with_capacity(rows.len());
    for row in rows {
        models.push(ModelRow {
            id: row.try_get("id")?,
            provider_id: row.try_get("provider_id")?,
            api_type: row.try_get("api_type")?,
            name: row.try_get("name")?,
            enabled: row.try_get("enabled")?,
            created_at: row.try_get("created_at")?,
        });
    }

    Ok(models)
}

pub struct CreateModelParams {
    pub provider_id: Uuid,
    pub api_type: ApiType,
    pub name: String,
}

pub async fn create_model(
    pool: &PgPool,
    params: CreateModelParams,
) -> anyhow::Result<ModelRow> {
    let row = sqlx::query(
        "INSERT INTO models (provider_id, api_type, name)
         VALUES ($1, $2, $3)
         RETURNING id, provider_id, api_type, name, enabled, created_at",
    )
    .bind(params.provider_id)
    .bind(params.api_type)
    .bind(params.name)
    .fetch_one(pool)
    .await?;

    Ok(ModelRow {
        id: row.try_get("id")?,
        provider_id: row.try_get("provider_id")?,
        api_type: row.try_get("api_type")?,
        name: row.try_get("name")?,
        enabled: row.try_get("enabled")?,
        created_at: row.try_get("created_at")?,
    })
}

pub struct UpdateModelParams {
    pub name: Option<String>,
    pub enabled: Option<bool>,
}

pub async fn update_model(
    pool: &PgPool,
    id: Uuid,
    params: UpdateModelParams,
) -> anyhow::Result<Option<ModelRow>> {
    let row = sqlx::query(
        "UPDATE models
         SET name = COALESCE($1, name),
             enabled = COALESCE($2, enabled)
         WHERE id = $3
         RETURNING id, provider_id, api_type, name, enabled, created_at",
    )
    .bind(params.name)
    .bind(params.enabled)
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(ModelRow {
        id: row.try_get("id")?,
        provider_id: row.try_get("provider_id")?,
        api_type: row.try_get("api_type")?,
        name: row.try_get("name")?,
        enabled: row.try_get("enabled")?,
        created_at: row.try_get("created_at")?,
    }))
}

pub async fn delete_model(pool: &PgPool, id: Uuid) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM models WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
