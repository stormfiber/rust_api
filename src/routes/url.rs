use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::utils::image::merge_json_body;

lazy_static! {
    static ref URL_STORE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

#[derive(Deserialize)]
pub struct UrlRequest {
    pub url: String,
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.route("/url/shorten", web::get().to(get_shorten));
    cfg.route("/url/shorten", web::post().to(post_shorten));
    cfg.route("/url/expand", web::get().to(get_expand));
    cfg.route("/url/expand", web::post().to(post_expand));
}

fn short_from_url(url: &str) -> String {
    format!("s{}", url.len())
}

// ----- GET -----

async fn get_shorten(
    web::Query(params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    let url = params.get("url").cloned().unwrap_or_default();
    if url.is_empty() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "url is required" }));
    }
    let short = short_from_url(&url);
    URL_STORE.lock().unwrap().insert(short.clone(), url);
    HttpResponse::Ok().json(serde_json::json!({ "short_url": short }))
}

async fn get_expand(
    web::Query(params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    let short = params.get("short").cloned().unwrap_or_default();
    let url = URL_STORE.lock().unwrap().get(&short).cloned();
    match url {
        Some(u) => HttpResponse::Ok().json(serde_json::json!({ "url": u })),
        None => HttpResponse::NotFound().json(serde_json::json!({ "error": "Not found" })),
    }
}

// ----- POST -----

async fn post_shorten(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    let url = params.get("url").cloned().unwrap_or_default();
    if url.is_empty() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "url is required" }));
    }
    let short = short_from_url(&url);
    URL_STORE.lock().unwrap().insert(short.clone(), url);
    HttpResponse::Ok().json(serde_json::json!({ "short_url": short }))
}

async fn post_expand(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    let short = params.get("short").cloned().unwrap_or_default();
    let url = URL_STORE.lock().unwrap().get(&short).cloned();
    match url {
        Some(u) => HttpResponse::Ok().json(serde_json::json!({ "url": u })),
        None => HttpResponse::NotFound().json(serde_json::json!({ "error": "Not found" })),
    }
}
