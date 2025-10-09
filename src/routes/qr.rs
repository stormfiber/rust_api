use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use qrcode::QrCode;
use qrcode::render::svg;
use image::{Luma};
use quircs::Quirc;
use std::io::Cursor;
use base64::{decode};

#[derive(Deserialize)]
struct QrRequest {
    text: String,
}

#[derive(Deserialize)]
struct QrDecodeRequest {
    base64_image: String,
}

#[post("/qr")]
async fn generate_qr(req: web::Json<QrRequest>) -> impl Responder {
    let code = QrCode::new(&req.text).unwrap();
    let image = code.render::<svg::Color>().min_dimensions(200, 200).build();
    HttpResponse::Ok().content_type("image/svg+xml").body(image)
}

#[post("/qr/png")]
async fn generate_qr_png(req: web::Json<QrRequest>) -> impl Responder {
    let code = QrCode::new(&req.text).unwrap();
    let img = code.render::<Luma<u8>>().min_dimensions(200, 200).build();
    let mut buffer = Cursor::new(Vec::new());
    img.save(&mut buffer, image::ImageOutputFormat::Png).unwrap();
    HttpResponse::Ok().content_type("image/png").body(buffer.into_inner())
}

#[post("/qr/decode")]
async fn decode_qr(req: web::Json<QrDecodeRequest>) -> impl Responder {
    let img_data = match decode(&req.base64_image) {
        Ok(data) => data,
        Err(_) => return HttpResponse::BadRequest().body("Invalid base64"),
};

    let img = image::load_from_memory(&img_data).unwrap().to_luma8();
    let mut quirc = Quirc::default();
    quirc.identify(img.width() as usize, img.height() as usize, &img);
    let codes = quirc.codes();
    let results: Vec<String> = codes.iter().map(|c| c.payload.clone()).collect();
    HttpResponse::Ok().json(results)
}

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(generate_qr);
    cfg.service(generate_qr_png);
    cfg.service(decode_qr);
}
