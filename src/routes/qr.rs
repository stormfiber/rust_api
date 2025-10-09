use actix_web::{web, HttpResponse, Responder};
use qrcode::QrCode;
use qrcode::render::unicode;
use image::{Luma, DynamicImage, ImageBuffer, ImageFormat};
use base64::{engine::general_purpose, Engine as _};
use std::collections::HashMap;
use std::io::Cursor;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.route("/qr/generate_unicode", web::get().to(generate_qr_unicode));
    cfg.route("/qr/generate_base64_png", web::post().to(generate_qr_base64_png));
}

// GET QR as Unicode blocks
async fn generate_qr_unicode(web::Query(query): web::Query<HashMap<String, String>>) -> impl Responder {
    let message = query.get("message").cloned().unwrap_or("Hello QR".to_string());

    let code = QrCode::new(message.clone()).unwrap();
    let string = code.render::<unicode::Dense1x2>().build();

    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(string)
}

// POST QR as Base64 PNG
async fn generate_qr_base64_png(web::Json(payload): web::Json<HashMap<String, String>>) -> impl Responder {
    let message = payload.get("message").cloned().unwrap_or("Hello QR".to_string());

    let code = QrCode::new(message.clone()).unwrap();

    // Render as image::Luma<u8>
    let qr_image = code.render::<Luma<u8>>().min_dimensions(200, 200).build();

    // Convert to DynamicImage
    let width = qr_image.width() as u32;
    let height = qr_image.height() as u32;
    let buffer: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, qr_image.into_vec()).unwrap();
    let dyn_image = DynamicImage::ImageLuma8(buffer);

    // Write to PNG in memory
    let mut png_bytes: Vec<u8> = Vec::new();
    dyn_image.write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png).unwrap();

    let base64_png = general_purpose::STANDARD.encode(&png_bytes);

    HttpResponse::Ok().json(serde_json::json!({
        "message": message,
        "qr_base64": base64_png
    }))
}
