use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Cursor;

use crate::utils::image::{self, merge_json_body};
use ::image::{DynamicImage, ImageFormat, Rgba};

const FONT_BYTES: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/DejaVuSans.ttf"));

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.route("/image/watermark", web::post().to(post_watermark));
}

async fn post_watermark(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_watermark(&params)
}

fn core_watermark(params: &HashMap<String, String>) -> impl Responder {
    let img = if let Some(b64) = params.get("image").filter(|s| !s.is_empty()) {
        match image::decode_base64_image(b64) {
            Ok(i) => i,
            Err(e) => return image::error_response(&e),
        }
    } else {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "image is required" }));
    };

    let text = params.get("text").cloned().unwrap_or_else(|| "WATERMARK".to_string());
    let position = params.get("position").cloned().unwrap_or_else(|| "bottom-right".to_string());
    let color = params.get("color").cloned().unwrap_or_else(|| "#ffffff".to_string());
    let opacity: f32 = params.get("opacity").and_then(|v| v.parse().ok()).unwrap_or(0.5);
    let size: u32 = params.get("size").and_then(|v| v.parse().ok()).unwrap_or(24);

    let mut out = img.to_rgba8();
    let (w, h) = (out.width(), out.height());

    let font = rusttype::Font::from_bytes(FONT_BYTES).unwrap();
    let scale = rusttype::Scale::uniform(size as f32);

    let mut text_w: u32 = 0;
    let mut text_h: u32 = 0;
    for ch in text.chars() {
        let g = font.glyph(ch);
        let bounded = g.scaled(scale).positioned(rusttype::point(0.0, 0.0));
        if let Some(bb) = bounded.pixel_bounding_box() {
            text_w = text_w.saturating_add(bb.width() as u32).saturating_add(2);
            text_h = text_h.max(bb.height() as u32);
        }
    }

    let (tx, ty) = match position.as_str() {
        "top-left" => (10u32, 10u32.saturating_add(text_h)),
        "top-right" => (w.saturating_sub(text_w).saturating_sub(10), 10u32.saturating_add(text_h)),
        "bottom-left" => (10u32, h.saturating_sub(text_h).saturating_sub(10)),
        "bottom-right" => (w.saturating_sub(text_w).saturating_sub(10), h.saturating_sub(5).max(5)),
        "center" => {
            let cx = w.saturating_sub(text_w) / 2;
            let cy = (h as i32 / 2 + text_h as i32 / 2).max(0) as u32;
            (cx, cy)
        }
        _ => (10u32, 10u32.saturating_add(text_h)),
    };

    let c = parse_color(&color);
    let a = (opacity.clamp(0.0, 1.0) * 255.0) as u32;
    let mut offset_x: u32 = tx;
    for ch in text.chars() {
        let g = font.glyph(ch);
        let bounded = g.scaled(scale).positioned(rusttype::point(0.0, 0.0));
        if let Some(bb) = bounded.pixel_bounding_box() {
            bounded.draw(|gx, gy, v| {
                let px = offset_x as i32 + gx as i32;
                let py = ty as i32 + bb.min.y + gy as i32;
                if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                    let p = out.get_pixel(px as u32, py as u32);
                    let inv = 255u32.saturating_sub((v * 255.0) as u32);
                    let blend_factor = inv * a / 255;
                    let blended = ((p.0[0] as u32 * (255 - blend_factor) / 255)
                        + (c.0[0] as u32 * blend_factor)) as u8;
                    out.put_pixel(px as u32, py as u32, Rgba([blended, p.0[1], p.0[2], p.0[3]]));
                }
            });
            offset_x = offset_x.saturating_add(bb.width() as u32).saturating_add(2);
        }
    }

    let mut buf = Vec::new();
    match DynamicImage::ImageRgba8(out).write_to(&mut Cursor::new(&mut buf), ImageFormat::Png) {
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
