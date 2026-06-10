use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use serde::Serialize;

use crate::app_state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_check))
        .route("/health/db", get(database_health_check))
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
    frontend_url: String,
}

#[derive(Serialize)]
struct DatabaseHealthResponse {
    status: &'static str,
    database: &'static str,
}

#[derive(Serialize)]
struct ErrorResponse {
    detail: String,
}

async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "sugoi-rec-rust-api",
        version: env!("CARGO_PKG_VERSION"),
        frontend_url: state.config.frontend_url,
    })
}

async fn database_health_check(
    State(state): State<AppState>,
) -> Result<Json<DatabaseHealthResponse>, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query("SELECT 1").execute(&state.db).await;

    match result {
        Ok(_) => Ok(Json(DatabaseHealthResponse {
            status: "ok",
            database: "postgres",
        })),
        Err(error) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                detail: format!("Database connection failed: {error}"),
            }),
        )),
    }
}
