use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct TextRequest {
    pub message: String,
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.route("/text/echo", web::get().to(echo));
    cfg.route("/text/reverse", web::post().to(reverse));
}

// GET /text/echo?message=Hello
async fn echo(web::Query(query): web::Query<HashMap<String, String>>) -> impl Responder {
    // Ensure the value is owned
    let message = query.get("message").cloned().unwrap_or_else(|| "Hello Text".to_string());
    HttpResponse::Ok().json(serde_json::json!({ "echo": message }))
}

// POST /text/reverse {"message": "Hello"}
async fn reverse(web::Json(payload): web::Json<TextRequest>) -> impl Responder {
    let reversed: String = payload.message.chars().rev().collect();
    HttpResponse::Ok().json(serde_json::json!({ "reversed": reversed }))
}
