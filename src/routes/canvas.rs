use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use std::collections::HashMap;

use crate::utils::image::{self, merge_json_body};
use ::image::{GenericImage, GenericImageView, ImageBuffer, Rgba};

const DEFAULT_WIDTH: u32 = 512;
const DEFAULT_HEIGHT: u32 = 512;
const DEFAULT_COLOR: &str = "#ffffff";
const DEFAULT_TEXT: &str = "Hello";
const DEFAULT_SIZE: u32 = 16;
const FONT_BYTES: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/DejaVuSans.ttf"));

#[derive(Deserialize, Clone, Default)]
pub struct CanvasPayload {
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default = "default_color_str")]
    pub color: String,
    #[serde(default = "default_text")]
    pub text: String,
    #[serde(default = "default_size")]
    pub size: u32,
    #[serde(default = "default_text_color")]
    pub text_color: String,
    #[serde(default)]
    pub x: Option<u32>,
    #[serde(default)]
    pub y: Option<u32>,
}

fn default_width() -> u32 {
    DEFAULT_WIDTH
}
fn default_height() -> u32 {
    DEFAULT_HEIGHT
}
fn default_color_str() -> String {
    DEFAULT_COLOR.to_string()
}
fn default_text() -> String {
    DEFAULT_TEXT.to_string()
}
fn default_size() -> u32 {
    DEFAULT_SIZE
}
fn default_text_color() -> String {
    "#000000".to_string()
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.route("/canvas/blank", web::get().to(get_blank));
    cfg.route("/canvas/blank", web::post().to(post_blank));
    cfg.route("/canvas/blank_base64", web::get().to(get_blank_base64));
    cfg.route(
        "/canvas/blank_base64",
        web::post().to(post_blank_base64),
    );
    cfg.route("/canvas/draw_text", web::get().to(get_draw_text));
    cfg.route("/canvas/draw_text", web::post().to(post_draw_text));
    cfg.route("/canvas/draw_text_base64", web::get().to(get_draw_text_base64));
    cfg.route(
        "/canvas/draw_text_base64",
        web::post().to(post_draw_text_base64),
    );
}

fn respond_png(result: ::image::DynamicImage) -> HttpResponse {
    match image::encode_png_bytes(&result) {
        Ok(bytes) => HttpResponse::Ok().content_type("image/png").body(bytes),
        Err(e) => image::error_response(&e),
    }
}

fn respond_base64(result: ::image::DynamicImage) -> HttpResponse {
    match image::encode_base64_png(&result) {
        Ok(b64) => HttpResponse::Ok().json(serde_json::json!({ "image_base64": b64 })),
        Err(e) => image::error_response(&e),
    }
}

fn parse_color(s: &str) -> Rgba<u8> {
    let s = s.trim_start_matches('#');
    let r = if s.len() >= 2 { u8::from_str_radix(&s[0..2], 16).unwrap_or(255) } else { 255 };
    let g = if s.len() >= 4 { u8::from_str_radix(&s[2..4], 16).unwrap_or(255) } else { 255 };
    let b = if s.len() >= 6 { u8::from_str_radix(&s[4..6], 16).unwrap_or(255) } else { 255 };
    Rgba([r, g, b, 255])
}

fn make_canvas(width: u32, height: u32, color: &str) -> ::image::DynamicImage {
    let w = width.max(1);
    let h = height.max(1);
    let c = parse_color(color);
    ::image::DynamicImage::ImageRgba8(ImageBuffer::from_pixel(w, h, c))
}

fn draw_text_on(
    canvas: &mut ::image::DynamicImage,
    text: &str,
    size: u32,
    color: &str,
    x: u32,
    y: u32,
) {
    let c = parse_color(color);
    let font = rusttype::Font::from_bytes(FONT_BYTES).unwrap();
    let scale = rusttype::Scale::uniform(size as f32);
    let mut offset_x = x;
    for ch in text.chars() {
        let g = font.glyph(ch);
        let bounded = g.scaled(scale).positioned(rusttype::point(0.0, 0.0));
            if let Some(bb) = bounded.pixel_bounding_box() {
                bounded.draw(|gx, gy, v| {
                    let px = offset_x as i32 + gx as i32;
                    let py = y as i32 + bb.min.y + gy as i32;
                    if px >= 0 && px < canvas.width() as i32 && py >= 0 && py < canvas.height() as i32 {
                        let p = canvas.get_pixel(px as u32, py as u32);
                        let inv = 255 - (v * 255.0) as u8;
                        let blended =
                            ((p.0[0] as u32 * inv as u32 / 255) + (c.0[0] as u32 * (255 - inv) as u32 / 255))
                                as u8;
                        canvas.put_pixel(px as u32, py as u32, Rgba([blended, p.0[1], p.0[2], p.0[3]]));
                    }
                });
                offset_x = offset_x.saturating_add(bb.width() as u32).saturating_add(2);
            }
    }
}

fn payload_from(params: &HashMap<String, String>, body: Option<&[u8]>) -> CanvasPayload {
    let mut p = CanvasPayload {
        width: 0,
        height: 0,
        color: DEFAULT_COLOR.to_string(),
        text: DEFAULT_TEXT.to_string(),
        size: DEFAULT_SIZE,
        text_color: "#000000".to_string(),
        x: Some(0),
        y: Some(0),
    };
    for (k, v) in params {
        match k.as_str() {
            "width" => { p.width = v.parse().unwrap_or(DEFAULT_WIDTH); }
            "height" => { p.height = v.parse().unwrap_or(DEFAULT_HEIGHT); }
            "color" => p.color = v.clone(),
            "text" => p.text = v.clone(),
            "size" => p.size = v.parse().unwrap_or(DEFAULT_SIZE),
            "text_color" => p.text_color = v.clone(),
            "x" => p.x = Some(v.parse().unwrap_or(0)),
            "y" => p.y = Some(v.parse().unwrap_or(0)),
            _ => {}
        }
    }
    if let Some(b) = body {
        if let Ok(val) = serde_json::from_slice::<serde_json::Value>(b) {
            if let serde_json::Value::Object(map) = val {
                for (k, v) in map {
                    let vs = match v {
                        serde_json::Value::String(s) => s.clone(),
                        _ => v.to_string(),
                    };
                    match k.as_str() {
                        "width" => { p.width = vs.parse().unwrap_or(DEFAULT_WIDTH); }
                        "height" => { p.height = vs.parse().unwrap_or(DEFAULT_HEIGHT); }
                        "color" => p.color = vs.clone(),
                        "text" => p.text = vs.clone(),
                        "size" => p.size = vs.parse().unwrap_or(DEFAULT_SIZE),
                        "text_color" => p.text_color = vs.clone(),
                        "x" => p.x = Some(vs.parse().unwrap_or(0)),
                        "y" => p.y = Some(vs.parse().unwrap_or(0)),
                        _ => {}
                    }
                }
            }
        }
    }
    p
}

// ----- blank -----

async fn get_blank(web::Query(params): web::Query<HashMap<String, String>>) -> impl Responder {
    core_blank(&params, None, false)
}

async fn post_blank(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_blank(&params, Some(&body), false)
}

async fn get_blank_base64(
    web::Query(params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    core_blank(&params, None, true)
}

async fn post_blank_base64(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_blank(&params, Some(&body), true)
}

fn core_blank(params: &HashMap<String, String>, body: Option<&[u8]>, as_base64: bool) -> impl Responder {
    let p = payload_from(params, body);
    if p.width == 0 || p.height == 0 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "width and height are required" }));
    }
    let img = make_canvas(p.width, p.height, &p.color);
    if as_base64 { respond_base64(img) } else { respond_png(img) }
}

// ----- draw_text -----

async fn get_draw_text(
    web::Query(params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    core_draw_text(&params, None, false)
}

async fn post_draw_text(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_draw_text(&params, Some(&body), false)
}

async fn get_draw_text_base64(
    web::Query(params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    core_draw_text(&params, None, true)
}

async fn post_draw_text_base64(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_draw_text(&params, Some(&body), true)
}

fn core_draw_text(params: &HashMap<String, String>, body: Option<&[u8]>, as_base64: bool) -> impl Responder {
    let p = payload_from(params, body);
    if p.width == 0 || p.height == 0 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "width and height are required" }));
    }
    let mut img = make_canvas(p.width, p.height, &p.color);
    let x = p.x.unwrap_or(10);
    let y = p.y.unwrap_or(p.size + 10);
    draw_text_on(&mut img, &p.text, p.size, &p.text_color, x, y);
    if as_base64 { respond_base64(img) } else { respond_png(img) }
}
