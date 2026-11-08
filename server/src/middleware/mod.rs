use crate::{AppState, error::AppError, repositories::users, services::auth};
use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use uuid::Uuid;
pub struct AuthUser(pub Uuid);
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or_else(AppError::unauthorized)?;
        let id = auth::verify_token(token, &state.config.secrets.jwt_secret)?;
        users::by_id(&state.pool, id)
            .await?
            .ok_or_else(AppError::unauthorized)?;
        Ok(Self(id))
    }
}
