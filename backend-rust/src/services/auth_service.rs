use axum::http::HeaderValue;
use sqlx::PgPool;

use crate::{
    config::Config,
    core::security::{create_access_token, decode_access_token, hash_password, verify_password},
    errors::AppError,
    repositories::user_repository,
    schemas::auth::{LoginForm, RegisterRequest, TokenResponse, UserResponse},
};

pub async fn register(db: &PgPool, payload: RegisterRequest) -> Result<UserResponse, AppError> {
    let email = normalize_email(&payload.email);

    if email.is_empty() {
        return Err(AppError::BadRequest("Email is required.".to_string()));
    }

    if payload.password.is_empty() {
        return Err(AppError::BadRequest("Password is required.".to_string()));
    }

    let existing_user = user_repository::find_by_email(db, &email).await?;

    if existing_user.is_some() {
        return Err(AppError::BadRequest(
            "Email already registered.".to_string(),
        ));
    }

    let password_hash = hash_password(&payload.password)?;

    let user = match user_repository::create_user(db, &email, &password_hash).await {
        Ok(user) => user,
        Err(error) => {
            if is_unique_violation(&error) {
                return Err(AppError::BadRequest(
                    "Email already registered.".to_string(),
                ));
            }

            return Err(error.into());
        }
    };

    Ok(UserResponse::from(user))
}

pub async fn login(
    db: &PgPool,
    config: &Config,
    payload: LoginForm,
) -> Result<TokenResponse, AppError> {
    let email = normalize_email(&payload.username);

    let user = user_repository::find_by_email(db, &email)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid credentials.".to_string()))?;

    if !verify_password(&payload.password, &user.password_hash) {
        return Err(AppError::Unauthorized("Invalid credentials.".to_string()));
    }

    let access_token = create_access_token(&user.id.to_string(), config)?;

    Ok(TokenResponse {
        access_token,
        token_type: "bearer",
    })
}

pub async fn current_user(
    db: &PgPool,
    config: &Config,
    authorization: Option<&HeaderValue>,
) -> Result<UserResponse, AppError> {
    let token = extract_bearer_token(authorization)?;
    let subject = decode_access_token(token, config)?;

    let user_id = subject
        .parse::<i32>()
        .map_err(|_| AppError::Unauthorized("Invalid token subject.".to_string()))?;

    let user = user_repository::find_by_id(db, user_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("User not found.".to_string()))?;

    Ok(UserResponse::from(user))
}

fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

fn extract_bearer_token(authorization: Option<&HeaderValue>) -> Result<&str, AppError> {
    let header = authorization
        .ok_or_else(|| AppError::Unauthorized("Missing authorization header.".to_string()))?;

    let value = header
        .to_str()
        .map_err(|_| AppError::Unauthorized("Invalid authorization header.".to_string()))?;

    value
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid authorization scheme.".to_string()))
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    match error {
        sqlx::Error::Database(database_error) => database_error.code().as_deref() == Some("23505"),
        _ => false,
    }
}
