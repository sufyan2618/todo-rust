use crate::models::{User, UserCredentials};
use sqlx::PgPool;
use uuid::Uuid;
pub async fn create(
    pool: &PgPool,
    name: &str,
    email: &str,
    hash: &str,
) -> Result<User, sqlx::Error> {
    sqlx::query_as("INSERT INTO users (id, name, email, password_hash) VALUES ($1,$2,$3,$4) RETURNING id,name,email")
        .bind(Uuid::new_v4()).bind(name).bind(email).bind(hash).fetch_one(pool).await
}
pub async fn by_email(pool: &PgPool, email: &str) -> Result<Option<UserCredentials>, sqlx::Error> {
    sqlx::query_as("SELECT id,name,email,password_hash FROM users WHERE email=$1")
        .bind(email)
        .fetch_optional(pool)
        .await
}
pub async fn by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as("SELECT id,name,email FROM users WHERE id=$1")
        .bind(id)
        .fetch_optional(pool)
        .await
}
