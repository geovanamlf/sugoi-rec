use axum::{
    extract::{Form, State},
    http::{header::AUTHORIZATION, HeaderMap},
    routing::{get, post},
    Json, Router,
};

use crate::{
    app_state::AppState,
    errors::AppError,
    schemas::auth::{LoginForm, RegisterRequest, TokenResponse, UserResponse},
    services::auth_service,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/me", get(me))
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

async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UserResponse>, AppError> {
    let authorization = headers.get(AUTHORIZATION);
    let user = auth_service::current_user(&state.db, &state.config, authorization).await?;
    Ok(Json(user))
}
