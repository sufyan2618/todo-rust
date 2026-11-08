use crate::error::AppError;
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    exp: usize,
    iss: String,
}
pub fn issue_token(user: Uuid, secret: &str) -> Result<String, AppError> {
    let claims = Claims {
        sub: user,
        exp: (Utc::now().timestamp() + 86400) as usize,
        iss: "todo-api".into(),
    };
    jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(AppError::internal)
}
pub fn verify_token(token: &str, secret: &str) -> Result<Uuid, AppError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&["todo-api"]);
    jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims.sub)
    .map_err(|_| AppError::unauthorized())
}
pub async fn hash_password(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
            .map(|hash| hash.to_string())
            .map_err(AppError::internal)
    })
    .await
    .map_err(AppError::internal)?
}
pub async fn verify_password(password: String, hash: String) -> Result<(), AppError> {
    tokio::task::spawn_blocking(move || {
        let parsed = PasswordHash::new(&hash).map_err(AppError::internal)?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .map_err(|_| AppError::unauthorized())
    })
    .await
    .map_err(AppError::internal)?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tokens_reject_wrong_keys_and_tampering() {
        let id = Uuid::new_v4();
        let token = issue_token(id, "a-long-test-secret-with-at-least-32-bytes").unwrap();
        assert_eq!(
            verify_token(&token, "a-long-test-secret-with-at-least-32-bytes").unwrap(),
            id
        );
        assert!(verify_token(&token, "different-secret").is_err());
        assert!(verify_token("invalid", "different-secret").is_err());
    }
    #[tokio::test]
    async fn passwords_are_salted_and_verified() {
        let hash = hash_password("password123".into()).await.unwrap();
        assert_ne!(hash, hash_password("password123".into()).await.unwrap());
        assert!(
            verify_password("password123".into(), hash.clone())
                .await
                .is_ok()
        );
        assert!(
            verify_password("wrong-password".into(), hash)
                .await
                .is_err()
        );
    }
}
