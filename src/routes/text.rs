use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use base64::{encode, decode};

#[derive(Deserialize)]
struct TextRequest {
    message: String,
}

#[post("/echo")]
async fn echo(req: web::Json<TextRequest>) -> impl Responder {
    HttpResponse::Ok().json(req.into_inner())
}

#[post("/reverse")]
async fn reverse(req: web::Json<TextRequest>) -> impl Responder {
    let reversed: String = req.message.chars().rev().collect();
    HttpResponse::Ok().json({ "reversed": reversed })
}

#[post("/base64/encode")]
async fn base64_encode(req: web::Json<TextRequest>) -> impl Responder {
    let encoded = encode(&req.message);
    HttpResponse::Ok().json({ "encoded": encoded })
}

#[post("/base64/decode")]
async fn base64_decode(req: web::Json<TextRequest>) -> impl Responder {
    match decode(&req.message) {
        Ok(bytes) => HttpResponse::Ok().json({ "decoded": String::from_utf8_lossy(&bytes) }),
        Err(_) => HttpResponse::BadRequest().body("Invalid Base64 string")
    }
}

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(echo);
    cfg.service(reverse);
    cfg.service(base64_encode);
    cfg.service(base64_decode);
}
