pub mod anime;
pub mod auth;
pub mod health;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};

use crate::app_state::AppState;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .merge(health::routes())
        .merge(auth::routes())
        .merge(anime::routes())
        .layer(cors)
        .with_state(state)
}
