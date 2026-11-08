use crate::{
    AppState,
    error::AppError,
    middleware::AuthUser,
    models::{CreateTodo, Todo, UpdateTodo},
    repositories::todos,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;
fn title(value: &str) -> Result<String, AppError> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 200 {
        return Err(AppError::bad_request("Title must contain 1–200 characters"));
    }
    Ok(value.to_string())
}
pub async fn list(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<Todo>>, AppError> {
    Ok(Json(todos::list(&state.pool, user).await?))
}
pub async fn create(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(input): Json<CreateTodo>,
) -> Result<(StatusCode, Json<Todo>), AppError> {
    Ok((
        StatusCode::CREATED,
        Json(todos::create(&state.pool, user, &title(&input.title)?).await?),
    ))
}
pub async fn update(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(mut input): Json<UpdateTodo>,
) -> Result<Json<Todo>, AppError> {
    if input.title.is_none() && input.completed.is_none() {
        return Err(AppError::bad_request("Provide a title or completed status"));
    }
    input.title = input.title.map(|value| title(&value)).transpose()?;
    Ok(Json(
        todos::update(&state.pool, user, id, input)
            .await?
            .ok_or_else(AppError::not_found)?,
    ))
}
pub async fn delete(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    if !todos::delete(&state.pool, user, id).await? {
        return Err(AppError::not_found());
    }
    Ok(StatusCode::NO_CONTENT)
}
#[cfg(test)]
mod tests {
    use super::title;
    #[test]
    fn validates_trimmed_titles() {
        assert_eq!(title("  Buy milk  ").unwrap_or_default(), "Buy milk");
        assert!(title("   ").is_err());
        assert!(title(&"x".repeat(201)).is_err());
    }
}
