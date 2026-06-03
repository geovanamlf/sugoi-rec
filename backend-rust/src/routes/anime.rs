use axum::{
    extract::{Path, Query, State},
    http::{header::AUTHORIZATION, HeaderMap},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use crate::{
    app_state::AppState,
    errors::AppError,
    schemas::anime::AnimeResponse,
    services::{anime_service, auth_service},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/anime/id/{anilist_id}", get(get_by_id))
        .route("/anime/search", get(search_by_name))
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
}

async fn get_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(anilist_id): Path<i32>,
) -> Result<Json<AnimeResponse>, AppError> {
    require_authenticated_user(&state, &headers).await?;

    let anime = anime_service::get_anime_by_anilist_id(&state, anilist_id).await?;
    Ok(Json(anime))
}

async fn search_by_name(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<SearchQuery>,
) -> Result<Json<AnimeResponse>, AppError> {
    require_authenticated_user(&state, &headers).await?;

    let anime = anime_service::get_anime_by_name(&state, &query.q).await?;
    Ok(Json(anime))
}

async fn require_authenticated_user(state: &AppState, headers: &HeaderMap) -> Result<(), AppError> {
    let authorization = headers.get(AUTHORIZATION);
    auth_service::current_user(&state.db, &state.config, authorization).await?;
    Ok(())
}
