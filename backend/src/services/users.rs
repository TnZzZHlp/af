use argon2::password_hash::{
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
};
use argon2::{Algorithm, Argon2, Params, Version};
use sqlx::{PgPool, types::time};
use uuid::Uuid;

use crate::db::users::{self, UserRow, fetch_user_by_username};

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub password_updated_at: Option<time::OffsetDateTime>,
    pub enabled: bool,
    pub created_at: time::OffsetDateTime,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        User {
            id: row.id,
            username: row.username,
            password_hash: row.password_hash,
            password_updated_at: row.password_updated_at,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

pub async fn create_user(pool: &PgPool, username: &str, password: &str) -> anyhow::Result<User> {
    let password_hash = hash_password(password)?;
    insert_user(pool, username, &password_hash).await
}

pub async fn authenticate_user(
    pool: &PgPool,
    username: &str,
    password: &str,
) -> anyhow::Result<Option<User>> {
    let Some(user) = fetch_user_by_username(pool, username).await? else {
        return Ok(None);
    };

    if verify_password(password, &user.password_hash)? {
        Ok(Some(user.into()))
    } else {
        Ok(None)
    }
}

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::default());
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|err| anyhow::anyhow!(err))?
        .to_string();
    Ok(hash)
}

pub fn verify_password(password: &str, password_hash: &str) -> anyhow::Result<bool> {
    let parsed_hash = PasswordHash::new(password_hash).map_err(|err| anyhow::anyhow!(err))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::default());
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub async fn insert_user(
    pool: &PgPool,
    username: &str,
    password_hash: &str,
) -> anyhow::Result<User> {
    let row = users::insert_user(pool, username, password_hash).await?;

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
) -> anyhow::Result<()> {
    users::update_password_hash(pool, user_id, password_hash).await
}

pub async fn list_users(pool: &PgPool) -> anyhow::Result<Vec<User>> {
    let users = users::list_users(pool).await?;
    Ok(users.into_iter().map(|u| u.into()).collect())
}

pub async fn get_user(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<User>> {
    let user = users::fetch_user_by_id(pool, id).await?;
    Ok(user.map(|u| u.into()))
}

pub async fn update_user(
    pool: &PgPool,
    id: Uuid,
    username: &str,
    enabled: bool,
) -> anyhow::Result<Option<User>> {
    let user = users::update_user(pool, id, username, enabled).await?;
    Ok(user.map(|u| u.into()))
}

pub async fn delete_user(pool: &PgPool, id: Uuid) -> anyhow::Result<()> {
    users::delete_user(pool, id).await
}

pub async fn change_password(pool: &PgPool, id: Uuid, password: &str) -> anyhow::Result<()> {
    let password_hash = hash_password(password)?;
    update_password_hash(pool, id, &password_hash).await
}
