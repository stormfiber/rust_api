use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;
use urlencoding::encode;

lazy_static! {
    static ref URL_STORE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

#[derive(Deserialize)]
struct UrlRequest {
    url: String,
}

#[post("/shorten")]
async fn shorten(req: web::Json<UrlRequest>) -> impl Responder {
    let short = format!("short.ly/{}", encode(&req.url)[..8].to_string());
    URL_STORE.lock().unwrap().insert(short.clone(), req.url.clone());
    HttpResponse::Ok().json({ "short_url": short })
}

#[derive(Deserialize)]
struct ShortRequest {
    short: String,
}

#[post("/expand")]
async fn expand(req: web::Json<ShortRequest>) -> impl Responder {
    match URL_STORE.lock().unwrap().get(&req.short) {
        Some(url) => HttpResponse::Ok().json({ "url": url.clone() }),
        None => HttpResponse::NotFound().body("Short URL not found"),
    }
}

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(shorten);
    cfg.service(expand);
}
