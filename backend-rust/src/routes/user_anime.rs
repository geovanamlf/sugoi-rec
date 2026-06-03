use axum::{
    extract::{Path, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    routing::{get, patch},
    Json, Router,
};

use crate::{
    app_state::AppState,
    errors::AppError,
    schemas::{
        auth::UserResponse,
        user_anime::{
            UserAnimeCreate, UserAnimeListItemResponse, UserAnimeResponse, UserAnimeUpdate,
        },
    },
    services::{auth_service, user_anime_service},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/list", get(list_anime).post(add_anime))
        .route("/list/", get(list_anime).post(add_anime))
        .route("/list/{anime_id}", patch(update_anime).delete(remove_anime))
}

async fn add_anime(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UserAnimeCreate>,
) -> Result<Json<UserAnimeResponse>, AppError> {
    let current_user = require_authenticated_user(&state, &headers).await?;

    let entry = user_anime_service::add_anime(&state.db, current_user.id, payload).await?;

    Ok(Json(entry))
}

async fn list_anime(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<UserAnimeListItemResponse>>, AppError> {
    let current_user = require_authenticated_user(&state, &headers).await?;

    let list = user_anime_service::list_anime(&state.db, current_user.id).await?;

    Ok(Json(list))
}

async fn update_anime(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(anime_id): Path<i32>,
    Json(payload): Json<UserAnimeUpdate>,
) -> Result<Json<UserAnimeResponse>, AppError> {
    let current_user = require_authenticated_user(&state, &headers).await?;

    let entry =
        user_anime_service::update_anime(&state.db, current_user.id, anime_id, payload).await?;

    Ok(Json(entry))
}

async fn remove_anime(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(anime_id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let current_user = require_authenticated_user(&state, &headers).await?;

    user_anime_service::remove_anime(&state.db, current_user.id, anime_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn require_authenticated_user(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<UserResponse, AppError> {
    let authorization = headers.get(AUTHORIZATION);
    auth_service::current_user(&state.db, &state.config, authorization).await
}
