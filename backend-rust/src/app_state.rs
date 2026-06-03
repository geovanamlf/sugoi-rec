use sqlx::PgPool;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: PgPool,
    pub http_client: reqwest::Client,
}

impl AppState {
    pub fn new(config: Config, db: PgPool, http_client: reqwest::Client) -> Self {
        Self {
            config,
            db,
            http_client,
        }
    }
}
