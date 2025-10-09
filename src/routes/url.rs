use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    static ref URL_STORE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

#[derive(Deserialize)]
pub struct UrlRequest {
    pub url: String,
}


pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.route("/url/shorten", web::post().to(shorten));
    cfg.route("/url/expand", web::get().to(expand));
}

// POST /url/shorten { "url": "https://example.com" }
async fn shorten(web::Json(payload): web::Json<UrlRequest>) -> impl Responder {
    let short = format!("s{}", payload.url.len()); // simple dummy short
    URL_STORE
        .lock()
        .unwrap()
        .insert(short.clone(), payload.url.clone());
    HttpResponse::Ok().json(serde_json::json!({ "short_url": short }))
}

// GET /url/expand?short=s18
async fn expand(web::Query(query): web::Query<HashMap<String, String>>) -> impl Responder {
    let short = query
        .get("short")
        .cloned() // clone the value so we own the string
        .unwrap_or_else(|| "".to_string()); // default empty string
    let url = URL_STORE.lock().unwrap().get(&short).cloned();
    match url {
        Some(u) => HttpResponse::Ok().json(serde_json::json!({ "url": u })),
        None => HttpResponse::NotFound().json(serde_json::json!({ "error": "Not found" })),
    }
}
