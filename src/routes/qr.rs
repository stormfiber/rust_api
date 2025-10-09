use actix_web::{web, HttpResponse, Responder};
use std::collections::HashMap;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.route("/qr/test", web::get().to(test));
    cfg.route("/qr/test_post", web::post().to(test_post));
}

// GET /qr/test?message=Hello
async fn test(web::Query(query): web::Query<HashMap<String, String>>) -> impl Responder {
    let message = query
        .get("message")
        .cloned() // clone the value so it owns the string
        .unwrap_or("Hello QR".to_string()); // default value
    HttpResponse::Ok().json(serde_json::json!({ "message": message }))
}

// POST /qr/test_post { "message": "Hello" }
async fn test_post(web::Json(payload): web::Json<HashMap<String, String>>) -> impl Responder {
    let message = payload
        .get("message")
        .cloned() // clone the value so it owns the string
        .unwrap_or("Hello QR".to_string()); // default value
    HttpResponse::Ok().json(serde_json::json!({ "message": message }))
}
