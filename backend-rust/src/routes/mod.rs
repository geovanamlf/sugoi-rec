pub mod analytics;
pub mod anime;
pub mod auth;
pub mod health;
pub mod recommendations;
pub mod user_anime;

use axum::{
    extract::DefaultBodyLimit,
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE, REFERRER_POLICY, X_CONTENT_TYPE_OPTIONS},
        HeaderName, HeaderValue, Method,
    },
    Router,
};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    set_header::SetResponseHeaderLayer,
};

use crate::app_state::AppState;

const MAX_REQUEST_BODY_SIZE: usize = 1024 * 1024;
const X_FRAME_OPTIONS: HeaderName = HeaderName::from_static("x-frame-options");

pub fn create_router(state: AppState) -> Router {
    let frontend_url = state.config.frontend_url.clone();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(move |origin, _request_parts| {
            origin.as_bytes() == frontend_url.as_bytes()
        }))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE]);

    Router::new()
        .merge(health::routes())
        .merge(auth::routes())
        .merge(anime::routes())
        .merge(user_anime::routes())
        .merge(analytics::routes())
        .merge(recommendations::routes())
        .layer(cors)
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BODY_SIZE))
        .layer(SetResponseHeaderLayer::if_not_present(
            X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            REFERRER_POLICY,
            HeaderValue::from_static("no-referrer"),
        ))
        .with_state(state)
}
