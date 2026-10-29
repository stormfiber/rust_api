use actix_web::{web, HttpResponse, Responder};
use qrcode::QrCode;
use ::image::{Luma, DynamicImage, ImageBuffer, ImageFormat};
use base64::{engine::general_purpose, Engine as _};
use std::collections::HashMap;
use std::io::Cursor;

use crate::utils::image::merge_json_body;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.route("/qr/generate_png", web::get().to(get_png));
    cfg.route("/qr/generate_png", web::post().to(post_png));
    cfg.route("/qr/generate_base64", web::get().to(get_base64));
    cfg.route("/qr/generate_base64", web::post().to(post_base64));
}

fn message_from_params(params: &HashMap<String, String>) -> String {
    params.get("message").cloned().unwrap_or_else(|| "Hello QR".to_string())
}

fn render_qr(message: &str) -> Vec<u8> {
    let code = QrCode::new(message).unwrap();
    let qr_image = code.render::<Luma<u8>>().min_dimensions(200, 200).build();
    let width = qr_image.width();
    let height = qr_image.height();
    let buffer: ImageBuffer<Luma<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, qr_image.into_vec()).unwrap();
    let dyn_image = DynamicImage::ImageLuma8(buffer);
    let mut png_bytes = Vec::new();
    dyn_image
        .write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)
        .unwrap();
    png_bytes
}

// ----- GET -----

async fn get_png(web::Query(params): web::Query<HashMap<String, String>>) -> impl Responder {
    let message = message_from_params(&params);
    let bytes = render_qr(&message);
    HttpResponse::Ok().content_type("image/png").body(bytes)
}

async fn get_base64(web::Query(params): web::Query<HashMap<String, String>>) -> impl Responder {
    let message = message_from_params(&params);
    let bytes = render_qr(&message);
    let base64_png = general_purpose::STANDARD.encode(&bytes);
    HttpResponse::Ok().json(serde_json::json!({
        "message": message,
        "qr_base64": base64_png
    }))
}

// ----- POST (JSON body or raw bytes) -----

async fn post_png(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    let message = message_from_params(&params);
    let bytes = render_qr(&message);
    HttpResponse::Ok().content_type("image/png").body(bytes)
}

async fn post_base64(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    let message = message_from_params(&params);
    let bytes = render_qr(&message);
    let base64_png = general_purpose::STANDARD.encode(&bytes);
    HttpResponse::Ok().json(serde_json::json!({
        "message": message,
        "qr_base64": base64_png
    }))
}
