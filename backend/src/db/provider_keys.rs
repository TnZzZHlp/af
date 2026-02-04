use sqlx::{types::time::OffsetDateTime, PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ProviderKeyRow {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub name: Option<String>,
    pub key: String,
    pub weight: i32,
    pub fail_count: i32,
    pub circuit_open_until: Option<OffsetDateTime>,
    pub last_fail_at: Option<OffsetDateTime>,
}

pub async fn fetch_provider_keys(
    pool: &PgPool,
    provider_id: Uuid,
) -> anyhow::Result<Vec<ProviderKeyRow>> {
    let rows = sqlx::query(
        "SELECT\n            id,\n            provider_id,\n            name,\n            key,\n            weight,\n            fail_count,\n            circuit_open_until,\n            last_fail_at\n         FROM provider_keys\n         WHERE provider_id = $1\n           AND enabled = true\n           AND (circuit_open_until IS NULL OR circuit_open_until <= now())\n         ORDER BY weight DESC, created_at ASC",
    )
    .bind(provider_id)
    .fetch_all(pool)
    .await?;

    let mut keys = Vec::with_capacity(rows.len());
    for row in rows {
        keys.push(ProviderKeyRow {
            id: row.try_get("id")?,
            provider_id: row.try_get("provider_id")?,
            name: row.try_get("name")?,
            key: row.try_get("key")?,
            weight: row.try_get("weight")?,
            fail_count: row.try_get("fail_count")?,
            circuit_open_until: row.try_get("circuit_open_until")?,
            last_fail_at: row.try_get("last_fail_at")?,
        });
    }

    Ok(keys)
}
