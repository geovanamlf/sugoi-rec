use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::{config::Config, errors::AppError};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    if password.len() > 72 {
        return Err(AppError::BadRequest(
            "Password too long (max 72 bytes).".to_string(),
        ));
    }

    hash(password, DEFAULT_COST).map_err(|error| {
        tracing::error!("Password hashing error: {error}");
        AppError::Internal("Could not hash password.".to_string())
    })
}

pub fn verify_password(password: &str, password_hash: &str) -> bool {
    verify(password, password_hash).unwrap_or(false)
}

pub fn create_access_token(subject: &str, config: &Config) -> Result<String, AppError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AppError::Internal("Invalid system time.".to_string()))?;

    let expiration = now + Duration::from_secs(config.jwt_access_token_expire_minutes * 60);

    let claims = Claims {
        sub: subject.to_string(),
        exp: expiration.as_secs() as usize,
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret_key.as_bytes()),
    )
    .map_err(|error| {
        tracing::error!("JWT encode error: {error}");
        AppError::Internal("Could not create access token.".to_string())
    })
}

pub fn decode_access_token(token: &str, config: &Config) -> Result<String, AppError> {
    let validation = Validation::new(Algorithm::HS256);

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret_key.as_bytes()),
        &validation,
    )
    .map_err(|_| AppError::Unauthorized("Invalid or expired token.".to_string()))?;

    Ok(token_data.claims.sub)
}
