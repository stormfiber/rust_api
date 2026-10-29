use std::env;

pub struct Config {
    pub port: u16,
    pub host: String,
    #[allow(dead_code)]
    pub max_payload_bytes: usize,
    #[allow(dead_code)]
    pub font_path: Option<String>,
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
