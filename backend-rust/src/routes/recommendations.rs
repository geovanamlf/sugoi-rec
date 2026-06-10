use axum::{
    extract::{Query, State},
    http::{header::AUTHORIZATION, HeaderMap},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use crate::{
    app_state::AppState,
    errors::AppError,
    schemas::{auth::UserResponse, recommendation::RecommendationResponse},
    services::{auth_service, recommendation_service},
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/recommendations/", get(get_recommendations))
}

#[derive(Debug, Deserialize)]
struct RecommendationsQuery {
    #[serde(default)]
    refresh: bool,
}

async fn get_recommendations(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<RecommendationsQuery>,
) -> Result<Json<Vec<RecommendationResponse>>, AppError> {
    let user = require_authenticated_user(&state, &headers).await?;

    let recommendations =
        recommendation_service::get_recommendations(&state, user.id, query.refresh).await?;

    Ok(Json(recommendations))
}

async fn require_authenticated_user(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<UserResponse, AppError> {
    let authorization = headers.get(AUTHORIZATION);
    auth_service::current_user(&state.db, &state.config, authorization).await
}
