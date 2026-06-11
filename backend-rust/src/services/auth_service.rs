use axum::http::HeaderValue;
use chrono::{Duration as ChronoDuration, Utc};
use sqlx::PgPool;

use crate::{
    config::Config,
    core::security::{
        access_token_expires_in_seconds, create_access_token, create_refresh_token,
        decode_access_token, hash_password, hash_refresh_token, verify_password,
    },
    errors::AppError,
    repositories::{refresh_token_repository, user_repository},
    schemas::auth::{LoginForm, RefreshTokenRequest, RegisterRequest, TokenResponse, UserResponse},
};

const WEAK_PASSWORDS: [&str; 12] = [
    "123456",
    "12345678",
    "123456789",
    "password",
    "password123",
    "senha",
    "senha123",
    "qwerty",
    "qwerty123",
    "admin",
    "admin123",
    "letmein",
];

const MAX_REFRESH_TOKEN_LENGTH: usize = 512;

pub async fn register(db: &PgPool, payload: RegisterRequest) -> Result<UserResponse, AppError> {
    let email = normalize_email(&payload.email);

    validate_email(&email)?;
    validate_password(&payload.password)?;

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

    validate_login_payload(&email, &payload.password)?;

    let user = user_repository::find_by_email(db, &email)
        .await?
        .ok_or_else(invalid_credentials)?;

    if !verify_password(&payload.password, &user.password_hash) {
        return Err(invalid_credentials());
    }

    issue_token_pair(db, config, user.id).await
}

pub async fn refresh_token(
    db: &PgPool,
    config: &Config,
    payload: RefreshTokenRequest,
) -> Result<TokenResponse, AppError> {
    validate_refresh_token_payload(&payload.refresh_token)?;

    let token_hash = hash_refresh_token(&payload.refresh_token);
    let now = Utc::now();

    let mut transaction = db.begin().await?;

    let stored_token =
        match refresh_token_repository::find_by_hash_for_update(&mut transaction, &token_hash)
            .await?
        {
            Some(token) => token,
            None => return Err(invalid_refresh_token()),
        };

    if stored_token.revoked_at.is_some() {
        refresh_token_repository::revoke_active_tokens_for_user(
            &mut transaction,
            stored_token.user_id,
        )
        .await?;

        transaction.commit().await?;

        tracing::warn!(
            user_id = stored_token.user_id,
            refresh_token_id = %stored_token.id,
            "Refresh token reuse detected. Active tokens for user were revoked."
        );

        return Err(invalid_refresh_token());
    }

    if stored_token.expires_at <= now {
        refresh_token_repository::revoke_by_id(&mut transaction, stored_token.id, None).await?;

        transaction.commit().await?;

        return Err(invalid_refresh_token());
    }

    let new_refresh_token = create_refresh_token();
    let new_refresh_token_hash = hash_refresh_token(&new_refresh_token);
    let new_refresh_token_expires_at =
        Utc::now() + ChronoDuration::days(config.refresh_token_expire_days);

    let new_stored_token = refresh_token_repository::create_in_transaction(
        &mut transaction,
        stored_token.user_id,
        &new_refresh_token_hash,
        new_refresh_token_expires_at,
    )
    .await?;

    refresh_token_repository::revoke_by_id(
        &mut transaction,
        stored_token.id,
        Some(new_stored_token.id),
    )
    .await?;

    transaction.commit().await?;

    let access_token = create_access_token(&stored_token.user_id.to_string(), config)?;

    Ok(TokenResponse {
        access_token,
        refresh_token: new_refresh_token,
        token_type: "bearer",
        expires_in: access_token_expires_in_seconds(config),
    })
}

pub async fn current_user(
    db: &PgPool,
    config: &Config,
    authorization: Option<&HeaderValue>,
) -> Result<UserResponse, AppError> {
    let token = extract_bearer_token(authorization)?;
    let subject = decode_access_token(token, config)?;

    let user_id = subject.parse::<i32>().map_err(|_| invalid_token())?;

    let user = user_repository::find_by_id(db, user_id)
        .await?
        .ok_or_else(invalid_token)?;

    Ok(UserResponse::from(user))
}

async fn issue_token_pair(
    db: &PgPool,
    config: &Config,
    user_id: i32,
) -> Result<TokenResponse, AppError> {
    let access_token = create_access_token(&user_id.to_string(), config)?;

    let refresh_token = create_refresh_token();
    let refresh_token_hash = hash_refresh_token(&refresh_token);
    let refresh_token_expires_at =
        Utc::now() + ChronoDuration::days(config.refresh_token_expire_days);

    refresh_token_repository::create(db, user_id, &refresh_token_hash, refresh_token_expires_at)
        .await?;

    Ok(TokenResponse {
        access_token,
        refresh_token,
        token_type: "bearer",
        expires_in: access_token_expires_in_seconds(config),
    })
}

fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

fn validate_email(email: &str) -> Result<(), AppError> {
    if email.is_empty() {
        return Err(AppError::BadRequest("Email is required.".to_string()));
    }

    if email.len() > 254 {
        return Err(AppError::BadRequest("Email is too long.".to_string()));
    }

    let has_single_at = email.matches('@').count() == 1;

    if !has_single_at {
        return Err(AppError::BadRequest("Invalid email format.".to_string()));
    }

    let (local_part, domain) = email
        .split_once('@')
        .ok_or_else(|| AppError::BadRequest("Invalid email format.".to_string()))?;

    if local_part.is_empty() || domain.is_empty() || !domain.contains('.') {
        return Err(AppError::BadRequest("Invalid email format.".to_string()));
    }

    Ok(())
}

fn validate_password(password: &str) -> Result<(), AppError> {
    if password.is_empty() {
        return Err(AppError::BadRequest("Password is required.".to_string()));
    }

    if password.len() < 8 {
        return Err(AppError::BadRequest(
            "Password must be at least 8 characters long.".to_string(),
        ));
    }

    if password.len() > 72 {
        return Err(AppError::BadRequest(
            "Password too long (max 72 bytes).".to_string(),
        ));
    }

    let normalized = password.trim().to_lowercase();

    if WEAK_PASSWORDS.contains(&normalized.as_str()) {
        return Err(AppError::BadRequest(
            "Password is too common or weak.".to_string(),
        ));
    }

    Ok(())
}

fn validate_login_payload(email: &str, password: &str) -> Result<(), AppError> {
    if email.is_empty() || password.is_empty() {
        return Err(invalid_credentials());
    }

    Ok(())
}

fn validate_refresh_token_payload(refresh_token: &str) -> Result<(), AppError> {
    if refresh_token.is_empty() {
        return Err(invalid_refresh_token());
    }

    if refresh_token.len() > MAX_REFRESH_TOKEN_LENGTH {
        return Err(invalid_refresh_token());
    }

    Ok(())
}

fn invalid_credentials() -> AppError {
    AppError::Unauthorized("Invalid credentials.".to_string())
}

fn invalid_token() -> AppError {
    AppError::Unauthorized("Invalid token.".to_string())
}

fn invalid_refresh_token() -> AppError {
    AppError::Unauthorized("Invalid refresh token.".to_string())
}

fn extract_bearer_token(authorization: Option<&HeaderValue>) -> Result<&str, AppError> {
    let header = authorization
        .ok_or_else(|| AppError::Unauthorized("Missing authorization header.".to_string()))?;

    let value = header
        .to_str()
        .map_err(|_| AppError::Unauthorized("Invalid authorization header.".to_string()))?;

    let mut parts = value.split_whitespace();

    let scheme = parts
        .next()
        .ok_or_else(|| AppError::Unauthorized("Invalid authorization scheme.".to_string()))?;

    if !scheme.eq_ignore_ascii_case("Bearer") {
        return Err(AppError::Unauthorized(
            "Invalid authorization scheme.".to_string(),
        ));
    }

    let token = parts
        .next()
        .ok_or_else(|| AppError::Unauthorized("Invalid authorization header.".to_string()))?;

    if parts.next().is_some() {
        return Err(AppError::Unauthorized(
            "Invalid authorization header.".to_string(),
        ));
    }

    Ok(token)
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    match error {
        sqlx::Error::Database(database_error) => database_error.code().as_deref() == Some("23505"),
        _ => false,
    }
}
