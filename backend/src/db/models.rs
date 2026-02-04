use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ModelRow {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub api_type: String,
    pub name: String,
}

pub async fn fetch_models(
    pool: &PgPool,
    provider_id: Uuid,
    api_type: &str,
) -> anyhow::Result<Vec<ModelRow>> {
    let rows = sqlx::query(
        "SELECT id, provider_id, api_type, name\n         FROM models\n         WHERE provider_id = $1\n           AND api_type = $2\n           AND enabled = true\n         ORDER BY name ASC",
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
        });
    }

    Ok(models)
}
