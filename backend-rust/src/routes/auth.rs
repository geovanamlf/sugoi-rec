use std::time::Duration;

use axum::{
    extract::{Form, State},
    http::{
        header::{AUTHORIZATION, COOKIE, SET_COOKIE},
        HeaderMap, HeaderValue, StatusCode,
    },
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use tower_governor::{errors::GovernorError, governor::GovernorConfigBuilder, GovernorLayer};

use crate::{
    app_state::AppState,
    config::Config,
    errors::AppError,
    schemas::auth::{LoginForm, RegisterRequest, TokenResponse, UserResponse},
    services::auth_service::{self, IssuedTokenPair},
};

const LOGIN_RATE_LIMIT_BURST: u32 = 5;
const LOGIN_RATE_LIMIT_REPLENISH_SECONDS: u64 = 12;

const REGISTER_RATE_LIMIT_BURST: u32 = 3;
const REGISTER_RATE_LIMIT_REPLENISH_SECONDS: u64 = 20;

const AUTH_RATE_LIMIT_MESSAGE: &str = "Too many authentication attempts. Please try again later.";

const REFRESH_TOKEN_COOKIE_NAME: &str = "sugoi_refresh_token";
const REFRESH_TOKEN_COOKIE_PATH: &str = "/auth";

pub fn routes() -> Router<AppState> {
    let login_rate_limit_config = GovernorConfigBuilder::default()
        .period(Duration::from_secs(LOGIN_RATE_LIMIT_REPLENISH_SECONDS))
        .burst_size(LOGIN_RATE_LIMIT_BURST)
        .finish()
        .expect("Failed to build login rate limit configuration.");

    let register_rate_limit_config = GovernorConfigBuilder::default()
        .period(Duration::from_secs(REGISTER_RATE_LIMIT_REPLENISH_SECONDS))
        .burst_size(REGISTER_RATE_LIMIT_BURST)
        .finish()
        .expect("Failed to build register rate limit configuration.");

    let register_routes = Router::new()
        .route("/auth/register", post(register))
        .route_layer(
            GovernorLayer::new(register_rate_limit_config)
                .error_handler(auth_rate_limit_error_handler),
        );

    let login_routes = Router::new().route("/auth/login", post(login)).route_layer(
        GovernorLayer::new(login_rate_limit_config).error_handler(auth_rate_limit_error_handler),
    );

    Router::new()
        .merge(register_routes)
        .merge(login_routes)
        .route("/auth/refresh", post(refresh))
        .route("/auth/logout", post(logout))
        .route("/auth/me", get(me))
}

fn auth_rate_limit_error_handler(error: GovernorError) -> Response {
    tracing::warn!("Authentication rate limit exceeded: {error}");

    AppError::TooManyRequests(AUTH_RATE_LIMIT_MESSAGE.to_string()).into_response()
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let user = auth_service::register(&state.db, payload).await?;
    Ok(Json(user))
}

async fn login(
    State(state): State<AppState>,
    Form(payload): Form<LoginForm>,
) -> Result<Response, AppError> {
    let token_pair = auth_service::login(&state.db, &state.config, payload).await?;
    Ok(token_response_with_refresh_cookie(
        &state.config,
        token_pair,
    ))
}

async fn refresh(State(state): State<AppState>, headers: HeaderMap) -> Result<Response, AppError> {
    let refresh_token = refresh_token_from_cookie(&headers)?;
    let token_pair = auth_service::refresh_token(&state.db, &state.config, refresh_token).await?;

    Ok(token_response_with_refresh_cookie(
        &state.config,
        token_pair,
    ))
}

async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Result<Response, AppError> {
    if let Ok(refresh_token) = refresh_token_from_cookie(&headers) {
        auth_service::logout(&state.db, refresh_token).await?;
    }

    let mut response = StatusCode::NO_CONTENT.into_response();
    response
        .headers_mut()
        .insert(SET_COOKIE, expired_refresh_token_cookie(&state.config));

    Ok(response)
}

async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UserResponse>, AppError> {
    let authorization = headers.get(AUTHORIZATION);
    let user = auth_service::current_user(&state.db, &state.config, authorization).await?;
    Ok(Json(user))
}

fn token_response_with_refresh_cookie(config: &Config, token_pair: IssuedTokenPair) -> Response {
    let body = Json(TokenResponse {
        access_token: token_pair.access_token,
        token_type: token_pair.token_type,
        expires_in: token_pair.expires_in,
    });

    let mut response = body.into_response();
    response.headers_mut().insert(
        SET_COOKIE,
        refresh_token_cookie(config, &token_pair.refresh_token),
    );

    response
}

fn refresh_token_from_cookie(headers: &HeaderMap) -> Result<String, AppError> {
    let cookie_header = headers
        .get(COOKIE)
        .ok_or_else(|| AppError::Unauthorized("Missing refresh token cookie.".to_string()))?;

    let cookie_value = cookie_header
        .to_str()
        .map_err(|_| AppError::Unauthorized("Invalid cookie header.".to_string()))?;

    for cookie in cookie_value.split(';') {
        let trimmed = cookie.trim();

        if let Some((name, value)) = trimmed.split_once('=') {
            if name == REFRESH_TOKEN_COOKIE_NAME && !value.is_empty() {
                return Ok(value.to_string());
            }
        }
    }

    Err(AppError::Unauthorized(
        "Missing refresh token cookie.".to_string(),
    ))
}

fn refresh_token_cookie(config: &Config, refresh_token: &str) -> HeaderValue {
    let max_age_seconds = config.refresh_token_expire_days * 24 * 60 * 60;
    let secure_attribute = if config.refresh_token_cookie_secure {
        "; Secure"
    } else {
        ""
    };

    let cookie = format!(
        "{}={}; HttpOnly; Path={}; Max-Age={}; SameSite=Lax{}",
        REFRESH_TOKEN_COOKIE_NAME,
        refresh_token,
        REFRESH_TOKEN_COOKIE_PATH,
        max_age_seconds,
        secure_attribute
    );

    cookie
        .parse::<HeaderValue>()
        .expect("refresh token cookie header must be valid")
}

fn expired_refresh_token_cookie(config: &Config) -> HeaderValue {
    let secure_attribute = if config.refresh_token_cookie_secure {
        "; Secure"
    } else {
        ""
    };

    let cookie = format!(
        "{}=; HttpOnly; Path={}; Max-Age=0; SameSite=Lax{}",
        REFRESH_TOKEN_COOKIE_NAME, REFRESH_TOKEN_COOKIE_PATH, secure_attribute
    );

    cookie
        .parse::<HeaderValue>()
        .expect("expired refresh token cookie header must be valid")
}
