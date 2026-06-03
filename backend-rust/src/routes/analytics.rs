use axum::{
    extract::State,
    http::{header::AUTHORIZATION, HeaderMap},
    routing::get,
    Json, Router,
};

use crate::{
    app_state::AppState,
    errors::AppError,
    schemas::{
        analytics::{GenreStatsResponse, RatingStatsResponse, StatusStatsResponse},
        auth::UserResponse,
    },
    services::{analytics_service, auth_service},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/analytics/genres", get(get_genres))
        .route("/analytics/ratings", get(get_ratings))
        .route("/analytics/status", get(get_status))
}

async fn get_genres(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<GenreStatsResponse>>, AppError> {
    let current_user = require_authenticated_user(&state, &headers).await?;

    let stats = analytics_service::get_genre_stats(&state.db, current_user.id).await?;

    Ok(Json(stats))
}

async fn get_ratings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<RatingStatsResponse>, AppError> {
    let current_user = require_authenticated_user(&state, &headers).await?;

    let stats = analytics_service::get_rating_stats(&state.db, current_user.id).await?;

    Ok(Json(stats))
}

async fn get_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<StatusStatsResponse>>, AppError> {
    let current_user = require_authenticated_user(&state, &headers).await?;

    let stats = analytics_service::get_status_stats(&state.db, current_user.id).await?;

    Ok(Json(stats))
}

async fn require_authenticated_user(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<UserResponse, AppError> {
    let authorization = headers.get(AUTHORIZATION);
    auth_service::current_user(&state.db, &state.config, authorization).await
}
