use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct GatewayKeyModel {
    pub gateway_key_id: Uuid,
    pub model: String,
}

pub async fn fetch_model_whitelist(
    pool: &PgPool,
    gateway_key_id: Uuid,
) -> anyhow::Result<Vec<String>> {
    let rows = sqlx::query(
        "SELECT model\n         FROM gateway_key_models\n         WHERE gateway_key_id = $1\n         ORDER BY model",
    )
    .bind(gateway_key_id)
    .fetch_all(pool)
    .await?;

    let mut models = Vec::with_capacity(rows.len());
    for row in rows {
        models.push(row.try_get("model")?);
    }

    Ok(models)
}

pub async fn is_model_allowed(
    pool: &PgPool,
    gateway_key_id: Uuid,
    model: &str,
) -> anyhow::Result<bool> {
    let row = sqlx::query(
        "SELECT 1\n         FROM gateway_key_models\n         WHERE gateway_key_id = $1 AND model = $2\n         LIMIT 1",
    )
    .bind(gateway_key_id)
    .bind(model)
    .fetch_optional(pool)
    .await?;

    Ok(row.is_some())
}
