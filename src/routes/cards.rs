use actix_web::{web, HttpResponse, Responder};
use std::collections::HashMap;

use crate::utils::image::{self, merge_json_body};
use ::image::{GenericImage, GenericImageView, ImageBuffer, Rgba};

const DEFAULT_WIDTH: u32 = 600;
const DEFAULT_HEIGHT: u32 = 300;
const FONT_BYTES: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/DejaVuSans.ttf"));

#[derive(Clone, Default)]
pub struct CardParams {
    pub card_type: String,
    pub text: String,
    pub title: String,
    pub subtitle: String,
    pub background: String,
    pub text_color: String,
    pub accent: String,
    pub width: u32,
    pub height: u32,
}

pub fn register(cfg: &mut web::ServiceConfig) {
    let types = [
        "welcome", "rank", "leave", "profile", "birthday", "achievement",
        "quote", "spotify", "github_stats", "warning", "ban", "announcement",
        "poll_vote", "ship", "magic8ball", "weather", "certificate",
        "support_ticket",
    ];
    for t in types {
        let base = format!("/cards/{t}");
        cfg.route(&base, web::get().to(card_get));
        cfg.route(&base, web::post().to(card_post));
        let b64 = format!("{base}_base64");
        cfg.route(&b64, web::get().to(card_base64_get));
        cfg.route(&b64, web::post().to(card_base64_post));
    }
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

fn card_params_from(params: &HashMap<String, String>, body: Option<&[u8]>) -> CardParams {
    let mut p = CardParams::default();
    for (k, v) in params {
        match k.as_str() {
            "text" => p.text = v.clone(),
            "title" => p.title = v.clone(),
            "subtitle" => p.subtitle = v.clone(),
            "background" => p.background = v.clone(),
            "text_color" => p.text_color = v.clone(),
            "accent" => p.accent = v.clone(),
            "width" => p.width = v.parse().unwrap_or(DEFAULT_WIDTH),
            "height" => p.height = v.parse().unwrap_or(DEFAULT_HEIGHT),
            _ => {}
        }
    }
    if let Some(b) = body {
        if let Ok(val) = serde_json::from_slice::<serde_json::Value>(b) {
            if let serde_json::Value::Object(map) = val {
                for (k, v) in map {
                    let vs = v.to_string();
                    match k.as_str() {
                        "text" => p.text = vs.clone(),
                        "title" => p.title = vs.clone(),
                        "subtitle" => p.subtitle = vs.clone(),
                        "background" => p.background = vs.clone(),
                        "text_color" => p.text_color = vs.clone(),
                        "accent" => p.accent = vs.clone(),
                        "width" => p.width = vs.parse().unwrap_or(DEFAULT_WIDTH),
                        "height" => p.height = vs.parse().unwrap_or(DEFAULT_HEIGHT),
                        _ => {}
                    }
                }
            }
        }
    }
    p
}

fn defaults_for(t: &str) -> CardParams {
    let mut p = CardParams {
        card_type: t.to_string(),
        width: DEFAULT_WIDTH,
        height: DEFAULT_HEIGHT,
        ..Default::default()
    };
    match t {
        "welcome" => {
            p.text = "Welcome aboard!".to_string();
            p.background = "#1a1a2e".to_string();
            p.text_color = "#00d2ff".to_string();
            p.accent = "#16213e".to_string();
        }
        "rank" => {
            p.title = "Rank Card".to_string();
            p.text = "Level 5".to_string();
            p.background = "#2d3436".to_string();
            p.text_color = "#ffffff".to_string();
            p.accent = "#0984e3".to_string();
        }
        "leave" => {
            p.text = "Goodbye!".to_string();
            p.background = "#d63031".to_string();
            p.text_color = "#ffffff".to_string();
            p.accent = "#bf360c".to_string();
        }
        "profile" => {
            p.title = "Profile".to_string();
            p.text = "User Name".to_string();
            p.background = "#636e72".to_string();
            p.text_color = "#ffffff".to_string();
            p.accent = "#00cec9".to_string();
        }
        "birthday" => {
            p.text = "Happy Birthday!".to_string();
            p.background = "#e84393".to_string();
            p.text_color = "#ffffff".to_string();
            p.accent = "#fd79a8".to_string();
        }
        "achievement" => {
            p.title = "Achievement Unlocked".to_string();
            p.text = "Well done!".to_string();
            p.background = "#00b894".to_string();
            p.text_color = "#ffffff".to_string();
            p.accent = "#fdcb6e".to_string();
        }
        "quote" => {
            p.text = "Hello world".to_string();
            p.background = "#1e1e2e".to_string();
            p.text_color = "#ffd700".to_string();
            p.accent = "#4b0082".to_string();
        }
        "spotify" => {
            p.title = "Now Playing".to_string();
            p.text = "Track Title".to_string();
            p.background = "#1db954".to_string();
            p.text_color = "#ffffff".to_string();
            p.accent = "#1ed760".to_string();
        }
        "github_stats" => {
            p.title = "GitHub Stats".to_string();
            p.text = "123 Repos".to_string();
            p.background = "#24292e".to_string();
            p.text_color = "#ffffff".to_string();
            p.accent = "#0366d6".to_string();
        }
        "warning" => {
            p.text = "Warning!".to_string();
            p.background = "#fdcb6e".to_string();
            p.text_color = "#d63031".to_string();
            p.accent = "#e17055".to_string();
        }
        "ban" => {
            p.text = "Banned".to_string();
            p.background = "#d63031".to_string();
            p.text_color = "#ffffff".to_string();
            p.accent = "#7f8c8d".to_string();
        }
        "announcement" => {
            p.text = "Announcement".to_string();
            p.background = "#0984e3".to_string();
            p.text_color = "#ffffff".to_string();
            p.accent = "#74b9ff".to_string();
        }
        "poll_vote" => {
            p.text = "Vote Cast".to_string();
            p.background = "#00b894".to_string();
            p.text_color = "#ffffff".to_string();
            p.accent = "#55efc4".to_string();
        }
        "ship" => {
            p.text = "Shipping!".to_string();
            p.background = "#6c5ce7".to_string();
            p.text_color = "#ffffff".to_string();
            p.accent = "#a29bfe".to_string();
        }
        "magic8ball" => {
            p.text = "Ask again later".to_string();
            p.background = "#2d3436".to_string();
            p.text_color = "#ffffff".to_string();
            p.accent = "#00cec9".to_string();
        }
        "weather" => {
            p.text = "Sunny 25°C".to_string();
            p.background = "#74b9ff".to_string();
            p.text_color = "#2d3436".to_string();
            p.accent = "#0984e3".to_string();
        }
        "certificate" => {
            p.title = "Certificate".to_string();
            p.text = "Awarded to".to_string();
            p.background = "#ffffff".to_string();
            p.text_color = "#2d3436".to_string();
            p.accent = "#fdcb6e".to_string();
        }
        "support_ticket" => {
            p.title = "Support Ticket".to_string();
            p.text = "Ticket #1234".to_string();
            p.background = "#636e72".to_string();
            p.text_color = "#ffffff".to_string();
            p.accent = "#00cec9".to_string();
        }
        _ => {}
    }
    p
}

fn draw_text_at(
    canvas: &mut ::image::DynamicImage,
    text: &str,
    size: u32,
    color: Rgba<u8>,
    x: u32,
    y: u32,
) -> u32 {
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
                            ((p.0[0] as u32 * inv as u32 / 255) + (color.0[0] as u32 * (255 - inv) as u32 / 255))
                                as u8;
                        canvas.put_pixel(px as u32, py as u32, Rgba([blended, p.0[1], p.0[2], p.0[3]]));
                    }
                });
                offset_x = offset_x.saturating_add(bb.width() as u32).saturating_add(2);
            }
    }
    offset_x
}

fn render_card(p: &CardParams) -> ::image::DynamicImage {
    let bg = parse_color(&p.background);
    let fg = parse_color(&p.text_color);
    let accent = parse_color(&p.accent);
    let w = p.width.max(100);
    let h = p.height.max(100);
    let mut canvas = ::image::DynamicImage::ImageRgba8(ImageBuffer::from_pixel(w, h, bg));

    let title_size = ((h as f32 * 0.18) as u32).max(16);
    let body_size = ((h as f32 * 0.12) as u32).max(12);

    let mut cy = (h / 4).max(20);
    if !p.title.is_empty() {
        draw_text_at(&mut canvas, &p.title, title_size, accent, 40, cy);
        cy += title_size + 10;
    }
    if !p.subtitle.is_empty() {
        draw_text_at(&mut canvas, &p.subtitle, body_size.max(10), fg, 40, cy);
        cy += body_size + 6;
    }
    if !p.text.is_empty() {
        draw_text_at(&mut canvas, &p.text, body_size, fg, 40, cy);
    }

    canvas
}

async fn card_get(web::Query(params): web::Query<HashMap<String, String>>) -> impl Responder {
    core_card(&params, None, false)
}

async fn card_post(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_card(&params, Some(&body), false)
}

async fn card_base64_get(web::Query(params): web::Query<HashMap<String, String>>) -> impl Responder {
    core_card(&params, None, true)
}

async fn card_base64_post(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_card(&params, Some(&body), true)
}

fn core_card(params: &HashMap<String, String>, body: Option<&[u8]>, as_base64: bool) -> impl Responder {
    let p0 = card_params_from(params, body);
    let card_type = params.get("card_type").cloned().unwrap_or_else(|| p0.card_type.clone());
    let mut p = defaults_for(&card_type);
    if !p0.text.is_empty() { p.text = p0.text; }
    if !p0.title.is_empty() { p.title = p0.title; }
    if !p0.subtitle.is_empty() { p.subtitle = p0.subtitle; }
    if !p0.background.is_empty() { p.background = p0.background; }
    if !p0.text_color.is_empty() { p.text_color = p0.text_color; }
    if !p0.accent.is_empty() { p.accent = p0.accent; }
    if p0.width != 0 { p.width = p0.width; }
    if p0.height != 0 { p.height = p0.height; }
    let img = render_card(&p);
    if as_base64 { respond_base64(img) } else { respond_png(img) }
}
