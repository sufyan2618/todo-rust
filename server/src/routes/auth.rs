use crate::{
    AppState,
    error::AppError,
    middleware::AuthUser,
    models::{AuthResponse, Login, Signup, User},
    repositories::users,
    services::auth,
};
use axum::{Json, extract::State, http::StatusCode};
use validator::Validate;
pub async fn signup(
    State(state): State<AppState>,
    Json(mut input): Json<Signup>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    input.name = input.name.trim().to_string();
    input.email = input.email.trim().to_lowercase();
    input.validate().map_err(|_| {
        AppError::bad_request(
            "Use a name (1–80 characters), valid email, and password (8–128 characters)",
        )
    })?;
    let hash = auth::hash_password(input.password).await?;
    let user = users::create(&state.pool, &input.name, &input.email, &hash)
        .await
        .map_err(|error| {
            if error
                .as_database_error()
                .is_some_and(|error| error.is_unique_violation())
            {
                AppError(
                    StatusCode::CONFLICT,
                    "An account with this email already exists".into(),
                )
            } else {
                AppError::internal(error)
            }
        })?;
    let token = auth::issue_token(user.id, &state.config.secrets.jwt_secret)?;
    Ok((StatusCode::CREATED, Json(AuthResponse { token, user })))
}
pub async fn login(
    State(state): State<AppState>,
    Json(mut input): Json<Login>,
) -> Result<Json<AuthResponse>, AppError> {
    input.email = input.email.trim().to_lowercase();
    input
        .validate()
        .map_err(|_| AppError::bad_request("Enter a valid email and password"))?;
    let user = users::by_email(&state.pool, &input.email)
        .await?
        .ok_or_else(AppError::unauthorized)?;
    auth::verify_password(input.password, user.password_hash).await?;
    let token = auth::issue_token(user.id, &state.config.secrets.jwt_secret)?;
    Ok(Json(AuthResponse {
        token,
        user: User {
            id: user.id,
            name: user.name,
            email: user.email,
        },
    }))
}
pub async fn me(
    State(state): State<AppState>,
    AuthUser(id): AuthUser,
) -> Result<Json<User>, AppError> {
    Ok(Json(
        users::by_id(&state.pool, id)
            .await?
            .ok_or_else(AppError::unauthorized)?,
    ))
}
