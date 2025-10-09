use actix_web::{post, web, HttpResponse, Responder};
use serde_json::Value;

#[derive(serde::Deserialize)]
struct JsonRequest {
    json: String,
}

#[post("/format")]
async fn format_json(req: web::Json<JsonRequest>) -> impl Responder {
    match serde_json::from_str::<Value>(&req.json) {
        Ok(v) => HttpResponse::Ok().json(serde_json::to_string_pretty(&v).unwrap()),
        Err(_) => HttpResponse::BadRequest().body("Invalid JSON"),
    }
}

#[post("/minify")]
async fn minify_json(req: web::Json<JsonRequest>) -> impl Responder {
    match serde_json::from_str::<Value>(&req.json) {
        Ok(v) => HttpResponse::Ok().json(serde_json::to_string(&v).unwrap()),
        Err(_) => HttpResponse::BadRequest().body("Invalid JSON"),
    }
}

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(format_json);
    cfg.service(minify_json);
}
