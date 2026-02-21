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

pub async fn replace_model_whitelist(
    pool: &PgPool,
    gateway_key_id: Uuid,
    models: &[String],
) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM gateway_key_models WHERE gateway_key_id = $1")
        .bind(gateway_key_id)
        .execute(&mut *tx)
        .await?;

    for model in models {
        sqlx::query("INSERT INTO gateway_key_models (gateway_key_id, model) VALUES ($1, $2)")
            .bind(gateway_key_id)
            .bind(model)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(())
}
