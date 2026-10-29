use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Cursor;

use crate::utils::image::{self, merge_json_body};
use ::image::{DynamicImage, ImageFormat, Rgba};

#[derive(Deserialize)]
pub struct CollagePayload {
    #[serde(default)]
    pub images: Vec<String>,
    #[serde(default = "default_cols")]
    pub cols: u32,
    #[serde(default = "default_bg")]
    pub background: String,
}

fn default_cols() -> u32 {
    2
}

fn default_bg() -> String {
    "#ffffff".to_string()
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.route("/image/collage", web::post().to(post_collage));
}

async fn post_collage(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_collage(&params)
}

fn core_collage(params: &HashMap<String, String>) -> impl Responder {
    let images_b64: Vec<String> = match params.get("images") {
        Some(v) => {
            if let Ok(arr) = serde_json::from_slice::<Vec<serde_json::Value>>(v.as_bytes()) {
                arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
            } else {
                v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
            }
        }
        None => Vec::new(),
    };
    let cols = params.get("cols").and_then(|v| v.parse().ok()).unwrap_or(2);
    let background = params.get("background").cloned().unwrap_or_else(|| "#ffffff".to_string());

    if images_b64.is_empty() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "images array is required" }));
    }

    let cols = cols.max(1);
    let rows = (images_b64.len() as u32 + cols - 1) / cols;

    let mut loaded: Vec<DynamicImage> = Vec::new();
    for b64 in &images_b64 {
        match image::decode_base64_image(b64) {
            Ok(img) => loaded.push(img),
            Err(e) => return image::error_response(&e),
        }
    }

    let max_w = loaded.iter().map(|i| i.width()).max().unwrap_or(100).max(1);
    let max_h = loaded.iter().map(|i| i.height()).max().unwrap_or(100).max(1);

    let bg = parse_color(&background);
    let canvas_w = max_w * cols;
    let canvas_h = max_h * rows;
    let mut canvas = ::image::ImageBuffer::from_pixel(canvas_w, canvas_h, bg);

    for (idx, img) in loaded.iter().enumerate() {
        let col = (idx as u32) % cols;
        let row = (idx as u32) / cols;
        let x = col * max_w;
        let y = row * max_h;
        ::image::imageops::overlay(&mut canvas, img, x as i64, y as i64);
    }

    let mut buf = Vec::new();
    match DynamicImage::ImageRgba8(canvas).write_to(&mut Cursor::new(&mut buf), ImageFormat::Png) {
        Ok(_) => HttpResponse::Ok().content_type("image/png").body(buf),
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": format!("encode failed: {e}") })),
    }
}

fn parse_color(s: &str) -> Rgba<u8> {
    let s = s.trim_start_matches('#');
    let r = if s.len() >= 2 { u8::from_str_radix(&s[0..2], 16).unwrap_or(255) } else { 255 };
    let g = if s.len() >= 4 { u8::from_str_radix(&s[2..4], 16).unwrap_or(255) } else { 255 };
    let b = if s.len() >= 6 { u8::from_str_radix(&s[4..6], 16).unwrap_or(255) } else { 255 };
    Rgba([r, g, b, 255])
}
