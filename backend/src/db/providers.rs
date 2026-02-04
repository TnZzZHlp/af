use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ProviderRow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

pub async fn fetch_provider(pool: &PgPool, name: &str) -> anyhow::Result<Option<ProviderRow>> {
    let row = sqlx::query(
        "SELECT id, name, description\n         FROM providers\n         WHERE name = $1 AND enabled = true\n         LIMIT 1",
    )
    .bind(name)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(ProviderRow {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
    }))
}
