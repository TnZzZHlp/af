use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AliasRow {
    pub id: Uuid,
    pub name: String,
    pub api_type: String,
    pub strategy: String,
}

pub async fn fetch_alias(
    pool: &PgPool,
    name: &str,
    api_type: &str,
) -> anyhow::Result<Option<AliasRow>> {
    let row = sqlx::query(
        "SELECT id, name, api_type, strategy\n         FROM aliases\n         WHERE name = $1 AND api_type = $2 AND enabled = true\n         LIMIT 1",
    )
    .bind(name)
    .bind(api_type)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(AliasRow {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        api_type: row.try_get("api_type")?,
        strategy: row.try_get("strategy")?,
    }))
}
