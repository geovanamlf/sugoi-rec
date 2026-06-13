use std::time::Duration;

use axum::{
    extract::{Form, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use tower_governor::{errors::GovernorError, governor::GovernorConfigBuilder, GovernorLayer};

use crate::{
    app_state::AppState,
    errors::AppError,
    schemas::auth::{
        LoginForm, LogoutRequest, RefreshTokenRequest, RegisterRequest, TokenResponse, UserResponse,
    },
    services::auth_service,
};

const LOGIN_RATE_LIMIT_BURST: u32 = 5;
const LOGIN_RATE_LIMIT_REPLENISH_SECONDS: u64 = 12;

const REGISTER_RATE_LIMIT_BURST: u32 = 3;
const REGISTER_RATE_LIMIT_REPLENISH_SECONDS: u64 = 20;

const AUTH_RATE_LIMIT_MESSAGE: &str = "Too many authentication attempts. Please try again later.";

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
) -> Result<Json<TokenResponse>, AppError> {
    let token = auth_service::login(&state.db, &state.config, payload).await?;
    Ok(Json(token))
}

async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    let token = auth_service::refresh_token(&state.db, &state.config, payload).await?;
    Ok(Json(token))
}

async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LogoutRequest>,
) -> Result<StatusCode, AppError> {
    auth_service::logout(
        &state.db,
        &state.config,
        headers.get(AUTHORIZATION),
        payload,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UserResponse>, AppError> {
    let authorization = headers.get(AUTHORIZATION);
    let user = auth_service::current_user(&state.db, &state.config, authorization).await?;
    Ok(Json(user))
}
