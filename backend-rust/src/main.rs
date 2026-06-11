mod app_state;
mod clients;
mod config;
mod core;
mod errors;
mod models;
mod repositories;
mod routes;
mod schemas;
mod services;

use std::{net::SocketAddr, time::Duration};

use app_state::AppState;
use config::Config;
use routes::create_router;

use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing::info;

const HTTP_CLIENT_TIMEOUT_SECONDS: u64 = 10;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let config = Config::from_env();
    let address = config.server_address();

    let db = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to PostgreSQL database.");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Failed to run database migrations.");

    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(HTTP_CLIENT_TIMEOUT_SECONDS))
        .build()
        .expect("Failed to build HTTP client.");

    let state = AppState::new(config, db, http_client);
    let app = create_router(state);

    let listener = TcpListener::bind(&address)
        .await
        .expect("Failed to bind TCP listener.");

    info!("Sugoi Rec Rust API running at http://{}", address);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Failed to start Axum server.");
}
