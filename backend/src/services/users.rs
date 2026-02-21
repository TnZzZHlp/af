use argon2::password_hash::{
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
};
use argon2::{Algorithm, Argon2, Params, Version};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::users::{self, User, fetch_user_by_username};
use crate::error::{AppError, AppResult};

pub async fn create_user(pool: &PgPool, username: &str, password: &str) -> AppResult<User> {
    let password_hash = hash_password(password)?;
    insert_user(pool, username, &password_hash).await
}

pub async fn authenticate_user(
    pool: &PgPool,
    username: &str,
    password: &str,
) -> AppResult<Option<User>> {
    let Some(user) = fetch_user_by_username(pool, username).await? else {
        return Ok(None);
    };

    if verify_password(password, &user.password_hash)? {
        Ok(Some(user))
    } else {
        Ok(None)
    }
}

pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::default());
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|err| AppError::Internal(anyhow::anyhow!(err.to_string())))?
        .to_string();
    Ok(hash)
}

pub fn verify_password(password: &str, password_hash: &str) -> AppResult<bool> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|err| AppError::Internal(anyhow::anyhow!(err.to_string())))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::default());
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub async fn insert_user(pool: &PgPool, username: &str, password_hash: &str) -> AppResult<User> {
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
) -> AppResult<()> {
    users::update_password_hash(pool, user_id, password_hash).await
}

pub async fn list_users(pool: &PgPool) -> AppResult<Vec<User>> {
    let users = users::list_users(pool).await?;
    Ok(users.into_iter().collect())
}

pub async fn get_user(pool: &PgPool, id: Uuid) -> AppResult<Option<User>> {
    let user = users::fetch_user_by_id(pool, id).await?;
    Ok(user)
}

pub async fn update_user(
    pool: &PgPool,
    id: Uuid,
    username: &str,
    enabled: bool,
) -> AppResult<Option<User>> {
    let user = users::update_user(pool, id, username, enabled).await?;
    Ok(user)
}

pub async fn delete_user(pool: &PgPool, id: Uuid) -> AppResult<()> {
    users::delete_user(pool, id).await
}

pub async fn change_password(pool: &PgPool, id: Uuid, password: &str) -> AppResult<()> {
    let password_hash = hash_password(password)?;
    update_password_hash(pool, id, &password_hash).await
}
