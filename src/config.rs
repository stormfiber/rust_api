use std::env;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub host: String,
    pub max_payload_bytes: usize,
    pub font_path: Option<String>,
    pub api_key: Option<String>,
    pub jwt_secret: String,
    pub redis_url: String,
    pub rate_limit_max: usize,
    pub rate_limit_window_secs: u64,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3000),
            max_payload_bytes: env::var("MAX_PAYLOAD_BYTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10 * 1024 * 1024),
            font_path: env::var("FONT_PATH").ok(),
            api_key: env::var("API_KEY").ok(),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "default-secret-change-me".to_string()),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            rate_limit_max: env::var("RATE_LIMIT_MAX")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            rate_limit_window_secs: env::var("RATE_LIMIT_WINDOW_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
        }
    }

    pub fn addr(&self) -> std::net::SocketAddr {
        format!("{}:{}", self.host, self.port).parse().unwrap()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from_env()
    }
}
