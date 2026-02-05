use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ProviderEndpointRow {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub api_type: String,
    pub url: String,
    pub weight: i32,
    pub priority: i32,
    pub timeout_ms: i32,
}

pub async fn fetch_provider_endpoints(
    pool: &PgPool,
    provider_id: Uuid,
    api_type: &str,
) -> anyhow::Result<Vec<ProviderEndpointRow>> {
    let rows = sqlx::query(
        "SELECT\n            id,\n            provider_id,\n            api_type,\n            url,\n            weight,\n            priority,\n            timeout_ms\n         FROM provider_endpoints\n         WHERE provider_id = $1\n           AND api_type = $2\n           AND enabled = true\n         ORDER BY priority DESC, weight DESC",
    )
    .bind(provider_id)
    .bind(api_type)
    .fetch_all(pool)
    .await?;

    let mut endpoints = Vec::with_capacity(rows.len());
    for row in rows {
        endpoints.push(ProviderEndpointRow {
            id: row.try_get("id")?,
            provider_id: row.try_get("provider_id")?,
            api_type: row.try_get("api_type")?,
            url: row.try_get("url")?,
            weight: row.try_get("weight")?,
            priority: row.try_get("priority")?,
            timeout_ms: row.try_get("timeout_ms")?,
        });
    }

    Ok(endpoints)
}
