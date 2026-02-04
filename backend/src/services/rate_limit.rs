use sqlx::PgPool;
use uuid::Uuid;

use crate::db::gateway_keys;

pub async fn fetch_limits(
    pool: &PgPool,
    gateway_key_id: Uuid,
) -> anyhow::Result<(Option<i32>, Option<i32>)> {
    gateway_keys::fetch_limits(pool, gateway_key_id).await
}
