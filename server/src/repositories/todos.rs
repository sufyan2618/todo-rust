use crate::models::{Todo, UpdateTodo};
use sqlx::PgPool;
use uuid::Uuid;
pub async fn list(pool: &PgPool, user: Uuid) -> Result<Vec<Todo>, sqlx::Error> {
    sqlx::query_as("SELECT id,title,completed,created_at,updated_at FROM todos WHERE user_id=$1 ORDER BY created_at DESC")
        .bind(user).fetch_all(pool).await
}
pub async fn create(pool: &PgPool, user: Uuid, title: &str) -> Result<Todo, sqlx::Error> {
    sqlx::query_as("INSERT INTO todos (id,user_id,title) VALUES ($1,$2,$3) RETURNING id,title,completed,created_at,updated_at")
        .bind(Uuid::new_v4()).bind(user).bind(title).fetch_one(pool).await
}
pub async fn update(
    pool: &PgPool,
    user: Uuid,
    id: Uuid,
    input: UpdateTodo,
) -> Result<Option<Todo>, sqlx::Error> {
    sqlx::query_as("UPDATE todos SET title=COALESCE($3,title),completed=COALESCE($4,completed),updated_at=now() WHERE id=$1 AND user_id=$2 RETURNING id,title,completed,created_at,updated_at")
        .bind(id).bind(user).bind(input.title).bind(input.completed).fetch_optional(pool).await
}
pub async fn delete(pool: &PgPool, user: Uuid, id: Uuid) -> Result<bool, sqlx::Error> {
    Ok(sqlx::query("DELETE FROM todos WHERE id=$1 AND user_id=$2")
        .bind(id)
        .bind(user)
        .execute(pool)
        .await?
        .rows_affected()
        == 1)
}
