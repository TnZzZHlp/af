use sqlx::{PgPool, types::time};
use uuid::Uuid;

use crate::error::AppResult;

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub password_updated_at: Option<time::OffsetDateTime>,
    pub enabled: bool,
    pub created_at: time::OffsetDateTime,
}

pub async fn fetch_user_by_username(pool: &PgPool, username: &str) -> AppResult<Option<User>> {
    let row = sqlx::query!(
        "SELECT id, username, password_hash, password_updated_at, enabled, created_at FROM users WHERE username = $1",
        username
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| User {
        id: row.id,
        username: row.username,
        password_hash: row.password_hash,
        password_updated_at: row.password_updated_at,
        enabled: row.enabled,
        created_at: row.created_at,
    }))
}

pub async fn insert_user(pool: &PgPool, username: &str, password_hash: &str) -> AppResult<User> {
    let row = sqlx::query!(
        "INSERT INTO users (username, password_hash) VALUES ($1, $2) RETURNING id, username, password_hash, password_updated_at, enabled, created_at",
        username,
        password_hash
    )
    .fetch_one(pool)
    .await?;

    Ok(User {
        id: row.id,
        username: row.username,
        password_hash: row.password_hash,
        password_updated_at: row.password_updated_at,
        enabled: row.enabled,
        created_at: row.created_at,
    })
}

pub async fn update_password_hash(
    pool: &PgPool,
    user_id: Uuid,
    password_hash: &str,
) -> AppResult<()> {
    sqlx::query!(
        "UPDATE users SET password_hash = $1, password_updated_at = now() WHERE id = $2",
        password_hash,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_users(pool: &PgPool) -> AppResult<Vec<User>> {
    let rows = sqlx::query!(
        "SELECT id, username, password_hash, password_updated_at, enabled, created_at
         FROM users
         ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;

    let mut users = Vec::new();
    for row in rows {
        users.push(User {
            id: row.id,
            username: row.username,
            password_hash: row.password_hash,
            password_updated_at: row.password_updated_at,
            enabled: row.enabled,
            created_at: row.created_at,
        });
    }

    Ok(users)
}

pub async fn fetch_user_by_id(pool: &PgPool, id: Uuid) -> AppResult<Option<User>> {
    let row = sqlx::query!(
        "SELECT id, username, password_hash, password_updated_at, enabled, created_at
         FROM users
         WHERE id = $1",
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| User {
        id: row.id,
        username: row.username,
        password_hash: row.password_hash,
        password_updated_at: row.password_updated_at,
        enabled: row.enabled,
        created_at: row.created_at,
    }))
}

pub async fn update_user(
    pool: &PgPool,
    id: Uuid,
    username: &str,
    enabled: bool,
) -> AppResult<Option<User>> {
    let row = sqlx::query!(
        "UPDATE users
         SET username = $1, enabled = $2
         WHERE id = $3
         RETURNING id, username, password_hash, password_updated_at, enabled, created_at",
        username,
        enabled,
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| User {
        id: row.id,
        username: row.username,
        password_hash: row.password_hash,
        password_updated_at: row.password_updated_at,
        enabled: row.enabled,
        created_at: row.created_at,
    }))
}

pub async fn delete_user(pool: &PgPool, id: Uuid) -> AppResult<()> {
    sqlx::query!("DELETE FROM users WHERE id = $1", id)
        .execute(pool)
        .await?;

    Ok(())
}
