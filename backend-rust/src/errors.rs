use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Unauthorized(String),
    Internal(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    detail: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, detail) = match self {
            AppError::BadRequest(detail) => (StatusCode::BAD_REQUEST, detail),
            AppError::Unauthorized(detail) => (StatusCode::UNAUTHORIZED, detail),
            AppError::Internal(detail) => (StatusCode::INTERNAL_SERVER_ERROR, detail),
        };

        (status, Json(ErrorResponse { detail })).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        tracing::error!("Database error: {error}");
        AppError::Internal("Database error.".to_string())
    }
}
