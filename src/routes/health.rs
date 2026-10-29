use actix_web::{web, HttpResponse, Responder};
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};

static START_TIME: AtomicU64 = AtomicU64::new(0);

pub fn init_start_time() {
    START_TIME.store(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(), Ordering::SeqCst);
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub uptime_secs: u64,
    pub version: String,
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(get_health));
}

async fn get_health() -> impl Responder {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let started = START_TIME.load(Ordering::SeqCst);
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
        uptime_secs: now.saturating_sub(started),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
