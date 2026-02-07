use sqlx::PgPool;
use uuid::Uuid;

pub async fn fetch_model_whitelist(
    pool: &PgPool,
    gateway_key_id: Uuid,
) -> anyhow::Result<Vec<String>> {
    let rows = sqlx::query!(
        "SELECT model FROM gateway_key_models WHERE gateway_key_id = $1 ORDER BY model",
        gateway_key_id
    )
    .fetch_all(pool)
    .await?;

    let mut models = Vec::with_capacity(rows.len());
    for row in rows {
        models.push(row.model);
    }

    Ok(models)
}
