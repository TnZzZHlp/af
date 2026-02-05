use sqlx::{PgPool, Row, types::time};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub password_hash: String,
    pub password_updated_at: Option<time::OffsetDateTime>,
    pub enabled: bool,
    pub created_at: time::OffsetDateTime,
}

pub async fn fetch_user_by_email(pool: &PgPool, email: &str) -> anyhow::Result<Option<UserRow>> {
    let row = sqlx::query(
        "SELECT id, email, name, password_hash, password_updated_at, enabled, created_at\n         FROM users\n         WHERE email = $1 AND enabled = true\n         LIMIT 1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(UserRow {
        id: row.try_get("id")?,
        email: row.try_get("email")?,
        name: row.try_get("name")?,
        password_hash: row.try_get("password_hash")?,
        password_updated_at: row.try_get("password_updated_at")?,
        enabled: row.try_get("enabled")?,
        created_at: row.try_get("created_at")?,
    }))
}

pub async fn insert_user(
    pool: &PgPool,
    email: &str,
    name: Option<&str>,
    password_hash: &str,
) -> anyhow::Result<UserRow> {
    let row = sqlx::query(
        "INSERT INTO users (email, name, password_hash)\n         VALUES ($1, $2, $3)\n         RETURNING id, email, name, password_hash, password_updated_at, enabled, created_at",
    )
    .bind(email)
    .bind(name)
    .bind(password_hash)
    .fetch_one(pool)
    .await?;

    Ok(UserRow {
        id: row.try_get("id")?,
        email: row.try_get("email")?,
        name: row.try_get("name")?,
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
