use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}
#[derive(sqlx::FromRow)]
pub struct UserCredentials {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password_hash: String,
}
#[derive(Deserialize, Validate)]
pub struct Signup {
    #[validate(length(min = 1, max = 80))]
    pub name: String,
    #[validate(email, length(max = 254))]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
}
#[derive(Deserialize, Validate)]
pub struct Login {
    #[validate(email, length(max = 254))]
    pub email: String,
    #[validate(length(min = 1, max = 128))]
    pub password: String,
}
#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}
#[derive(Serialize, sqlx::FromRow)]
pub struct Todo {
    pub id: Uuid,
    pub title: String,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[derive(Deserialize)]
pub struct CreateTodo {
    pub title: String,
}
#[derive(Deserialize)]
pub struct UpdateTodo {
    pub title: Option<String>,
    pub completed: Option<bool>,
}
