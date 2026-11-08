pub mod auth;
pub mod todos;
use crate::AppState;
use axum::{Json, extract::State, http::StatusCode};
use serde_json::{Value, json};
pub async fn health(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"status":"ok", "database":"connected"})),
        ),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"status":"unavailable", "database":"disconnected"})),
        ),
    }
}
