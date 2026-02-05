use sqlx::{PgPool, Row, types::time};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub password_updated_at: Option<time::OffsetDateTime>,
    pub enabled: bool,
    pub created_at: time::OffsetDateTime,
}



pub async fn fetch_user_by_username(
    pool: &PgPool,
    username: &str,
) -> anyhow::Result<Option<UserRow>> {
    let row = sqlx::query(
        "SELECT id, username, password_hash, password_updated_at, enabled, created_at\n         FROM users\n         WHERE username = $1",
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| UserRow {
        id: row.try_get("id").unwrap(),
        username: row.try_get("username").unwrap(),
        password_hash: row.try_get("password_hash").unwrap(),
        password_updated_at: row.try_get("password_updated_at").unwrap(),
        enabled: row.try_get("enabled").unwrap(),
        created_at: row.try_get("created_at").unwrap(),
    }))
}

pub async fn insert_user(
    pool: &PgPool,
    username: &str,
    password_hash: &str,
) -> anyhow::Result<UserRow> {
    let row = sqlx::query(
        "INSERT INTO users (username, password_hash)\n         VALUES ($1, $2)\n         RETURNING id, username, password_hash, password_updated_at, enabled, created_at",
    )
    .bind(username)
    .bind(password_hash)
    .fetch_one(pool)
    .await?;

    Ok(UserRow {
        id: row.try_get("id")?,
        username: row.try_get("username")?,
        password_hash: row.try_get("password_hash")?,
        password_updated_at: row.try_get("password_updated_at")?,
        enabled: row.try_get("enabled")?,
        created_at: row.try_get("created_at")?,
    })
}

pub async fn update_password_hash(
    pool: &PgPool,
    user_id: Uuid,
    password_hash: &str,
) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE users\n         SET password_hash = $1, password_updated_at = now()\n         WHERE id = $2",
    )
    .bind(password_hash)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}
