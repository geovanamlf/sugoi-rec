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
    pub refresh_token_expire_days: i64,
    pub refresh_token_cookie_secure: bool,
    pub anilist_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        let port = env::var("PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(8080);

        let frontend_url =
            env::var("FRONTEND_URL").unwrap_or_else(|_| "http://127.0.0.1:5173".to_string());

        let database_url =
            env::var("DATABASE_URL").expect("DATABASE_URL must be set in backend-rust/.env");

        let database_max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(5);

        let jwt_secret_key =
            env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set in backend-rust/.env");

        validate_jwt_secret_key(&jwt_secret_key);

        let jwt_access_token_expire_minutes = env::var("JWT_ACCESS_TOKEN_EXPIRE_MINUTES")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(15);

        validate_jwt_access_token_expire_minutes(jwt_access_token_expire_minutes);

        let refresh_token_expire_days = env::var("REFRESH_TOKEN_EXPIRE_DAYS")
            .ok()
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or(7);

        validate_refresh_token_expire_days(refresh_token_expire_days);

        let refresh_token_cookie_secure = env::var("REFRESH_TOKEN_COOKIE_SECURE")
            .ok()
            .and_then(|value| value.parse::<bool>().ok())
            .unwrap_or(false);

        let anilist_url =
            env::var("ANILIST_URL").unwrap_or_else(|_| "https://graphql.anilist.co".to_string());

        Self {
            host,
            port,
            frontend_url,
            database_url,
            database_max_connections,
            jwt_secret_key,
            jwt_access_token_expire_minutes,
            refresh_token_expire_days,
            refresh_token_cookie_secure,
            anilist_url,
        }
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

fn validate_jwt_secret_key(secret: &str) {
    let trimmed = secret.trim();
    let normalized = trimmed.to_lowercase();

    let weak_exact_values = ["change-me", "changeme", "secret", "development", "dev"];

    let weak_fragments = [
        "change-me",
        "changeme",
        "replace-with",
        "your-secret",
        "your_secret",
        "jwt-secret",
        "jwt_secret",
        "example",
        "placeholder",
    ];

    if weak_exact_values.contains(&normalized.as_str())
        || weak_fragments
            .iter()
            .any(|fragment| normalized.contains(fragment))
    {
        panic!("JWT_SECRET_KEY is too weak. Use a strong random secret.");
    }

    if trimmed.len() < 32 {
        panic!("JWT_SECRET_KEY is too short. Use at least 32 characters.");
    }
}

fn validate_jwt_access_token_expire_minutes(minutes: u64) {
    const MIN_MINUTES: u64 = 5;
    const MAX_MINUTES: u64 = 1440;

    if !(MIN_MINUTES..=MAX_MINUTES).contains(&minutes) {
        panic!("JWT_ACCESS_TOKEN_EXPIRE_MINUTES must be between 5 and 1440 minutes.");
    }
}

fn validate_refresh_token_expire_days(days: i64) {
    const MIN_DAYS: i64 = 1;
    const MAX_DAYS: i64 = 30;

    if !(MIN_DAYS..=MAX_DAYS).contains(&days) {
        panic!("REFRESH_TOKEN_EXPIRE_DAYS must be between 1 and 30 days.");
    }
}
