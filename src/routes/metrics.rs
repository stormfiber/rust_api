use actix_web::{web, HttpResponse, Responder};
use prometheus::{Encoder, Histogram, HistogramOpts, IntCounter, TextEncoder};
use std::sync::OnceLock;
use std::time::Instant;

fn requests_total() -> &'static IntCounter {
    static C: OnceLock<IntCounter> = OnceLock::new();
    C.get_or_init(|| {
        prometheus::register_int_counter!(
            "http_requests_total",
            "Total HTTP requests",
        ).unwrap()
    })
}

fn request_duration() -> &'static Histogram {
    static H: OnceLock<Histogram> = OnceLock::new();
    H.get_or_init(|| {
        prometheus::register_histogram!(
            HistogramOpts::new(
                "http_request_duration_seconds",
                "HTTP request duration in seconds",
            ),
        ).unwrap()
    })
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.route("/metrics", web::get().to(get_metrics));
}

async fn get_metrics() -> impl Responder {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buf = Vec::new();
    encoder.encode(&metric_families, &mut buf).unwrap();
    HttpResponse::Ok()
        .content_type(encoder.format_type())
        .body(buf)
}

pub fn record_request(_method: &str, _path: &str, status: u16, duration_secs: f64) {
    let _ = status;
    requests_total().inc();
    request_duration().observe(duration_secs);
}

pub struct Timer {
    start: Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self { start: Instant::now() }
    }

    pub fn elapsed_secs(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }
}
