use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub frontend_url: String,
    pub database_url: String,
    pub database_max_connections: u32,
    pub jwt_secret_key: String,
    pub jwt_access_token_expire_minutes: u64,
}

impl Config {
    pub fn from_env() -> Self {
        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        let port = env::var("PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(8080);

        let frontend_url =
            env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());

        let database_url =
            env::var("DATABASE_URL").expect("DATABASE_URL must be set in backend-rust/.env");

        let database_max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(5);

        let jwt_secret_key =
            env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set in backend-rust/.env");

        let jwt_access_token_expire_minutes = env::var("JWT_ACCESS_TOKEN_EXPIRE_MINUTES")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(30);

        Self {
            host,
            port,
            frontend_url,
            database_url,
            database_max_connections,
            jwt_secret_key,
            jwt_access_token_expire_minutes,
        }
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
