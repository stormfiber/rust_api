use actix_web::{web, HttpResponse, Responder};
use qrcode::QrCode;
use image::{Luma, DynamicImage, ImageBuffer, ImageFormat};
use base64::{engine::general_purpose, Engine as _};
use std::collections::HashMap;
use std::io::Cursor;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.route("/qr/generate_png", web::get().to(generate_qr_png));
    cfg.route("/qr/generate_base64", web::post().to(generate_qr_base64));
}

// GET QR as PNG image (for browser)
async fn generate_qr_png(web::Query(query): web::Query<HashMap<String, String>>) -> impl Responder {
    let message = query.get("message").cloned().unwrap_or("Hello QR".to_string());

    let code = QrCode::new(message.clone()).unwrap();
    let qr_image = code.render::<Luma<u8>>().min_dimensions(200, 200).build();

    let width = qr_image.width();
    let height = qr_image.height();
    let buffer: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, qr_image.into_vec()).unwrap();
    let dyn_image = DynamicImage::ImageLuma8(buffer);

    let mut png_bytes = Vec::new();
    dyn_image.write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png).unwrap();

    HttpResponse::Ok()
        .content_type("image/png")
        .body(png_bytes)
}

// POST QR as Base64 PNG in JSON
async fn generate_qr_base64(web::Json(payload): web::Json<HashMap<String, String>>) -> impl Responder {
    let message = payload.get("message").cloned().unwrap_or("Hello QR".to_string());

    let code = QrCode::new(message.clone()).unwrap();
    let qr_image = code.render::<Luma<u8>>().min_dimensions(200, 200).build();

    let width = qr_image.width();
    let height = qr_image.height();
    let buffer: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, qr_image.into_vec()).unwrap();
    let dyn_image = DynamicImage::ImageLuma8(buffer);

    let mut png_bytes = Vec::new();
    dyn_image.write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png).unwrap();

    let base64_png = general_purpose::STANDARD.encode(&png_bytes);

    HttpResponse::Ok().json(serde_json::json!({
        "message": message,
        "qr_base64": base64_png
    }))
}
