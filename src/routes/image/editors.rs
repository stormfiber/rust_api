use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use std::collections::HashMap;

use crate::utils::image::{self, merge_json_body};
use ::image::{ImageBuffer, Rgba};
use ::image::imageops;

#[derive(Deserialize)]
pub struct ResizeRequest {
    pub image: String,
    pub width: u32,
    pub height: u32,
    #[serde(default = "default_mode")]
    pub mode: String,
}

fn default_mode() -> String {
    "fit".to_string()
}

#[derive(Deserialize)]
pub struct RotateRequest {
    pub image: String,
    #[serde(default = "default_deg")]
    pub deg: u32,
}

fn default_deg() -> u32 {
    90
}

#[derive(Deserialize)]
pub struct FlipRequest {
    pub image: String,
    #[serde(default = "default_direction")]
    pub direction: String,
}

fn default_direction() -> String {
    "horizontal".to_string()
}

#[derive(Deserialize)]
pub struct BlurRequest {
    pub image: String,
    #[serde(default = "default_sigma")]
    pub sigma: f32,
}

fn default_sigma() -> f32 {
    2.0
}

pub fn register(cfg: &mut web::ServiceConfig) {
    // resize
    cfg.route("/image/editors/resize", web::get().to(get_resize));
    cfg.route("/image/editors/resize", web::post().to(post_resize));
    cfg.route(
        "/image/editors/resize_base64",
        web::get().to(get_resize_base64),
    );
    cfg.route(
        "/image/editors/resize_base64",
        web::post().to(post_resize_base64),
    );
    // rotate
    cfg.route("/image/editors/rotate", web::get().to(get_rotate));
    cfg.route("/image/editors/rotate", web::post().to(post_rotate));
    cfg.route(
        "/image/editors/rotate_base64",
        web::get().to(get_rotate_base64),
    );
    cfg.route(
        "/image/editors/rotate_base64",
        web::post().to(post_rotate_base64),
    );
    // flip
    cfg.route("/image/editors/flip", web::get().to(get_flip));
    cfg.route("/image/editors/flip", web::post().to(post_flip));
    cfg.route("/image/editors/flip_base64", web::get().to(get_flip_base64));
    cfg.route(
        "/image/editors/flip_base64",
        web::post().to(post_flip_base64),
    );
    // grayscale
    cfg.route(
        "/image/editors/grayscale",
        web::get().to(get_grayscale),
    );
    cfg.route(
        "/image/editors/grayscale",
        web::post().to(post_grayscale),
    );
    cfg.route(
        "/image/editors/grayscale_base64",
        web::get().to(get_grayscale_base64),
    );
    cfg.route(
        "/image/editors/grayscale_base64",
        web::post().to(post_grayscale_base64),
    );
    // blur
    cfg.route("/image/editors/blur", web::get().to(get_blur));
    cfg.route("/image/editors/blur", web::post().to(post_blur));
    cfg.route("/image/editors/blur_base64", web::get().to(get_blur_base64));
    cfg.route(
        "/image/editors/blur_base64",
        web::post().to(post_blur_base64),
    );
}

// ---------- shared core ----------

fn to_rgba8(img: &::image::DynamicImage) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    img.to_rgba8()
}

fn apply_resize(img: &::image::DynamicImage, width: u32, height: u32, mode: &str) -> ::image::DynamicImage {
    let _ = mode;
    let rgba = to_rgba8(img);
    ::image::DynamicImage::ImageRgba8(imageops::resize(
        &rgba,
        width,
        height,
        imageops::FilterType::Triangle,
    ))
}

fn apply_rotate(img: &::image::DynamicImage, deg: u32) -> ::image::DynamicImage {
    let rgba = to_rgba8(img);
    let rotated = match deg % 360 {
        90 => imageops::rotate90(&rgba),
        180 => imageops::rotate180(&rgba),
        270 => imageops::rotate270(&rgba),
        _ => rgba.clone(),
    };
    ::image::DynamicImage::ImageRgba8(rotated)
}

fn apply_flip(img: &::image::DynamicImage, direction: &str) -> ::image::DynamicImage {
    let rgba = to_rgba8(img);
    let flipped = match direction {
        "vertical" => imageops::flip_vertical(&rgba),
        _ => imageops::flip_horizontal(&rgba),
    };
    ::image::DynamicImage::ImageRgba8(flipped)
}

fn apply_grayscale(img: &::image::DynamicImage) -> ::image::DynamicImage {
    ::image::DynamicImage::ImageLuma8(imageops::grayscale(img))
}

fn apply_blur(img: &::image::DynamicImage, sigma: f32) -> ::image::DynamicImage {
    let rgba = to_rgba8(img);
    let blurred = imageproc::filter::gaussian_blur_f32(&rgba, sigma);
    ::image::DynamicImage::ImageRgba8(blurred)
}

fn load_input(params: &HashMap<String, String>, body: Option<&[u8]>) -> Result<::image::DynamicImage, crate::utils::image::ImageError> {
    if let Some(b64) = params.get("image").filter(|s| !s.is_empty()) {
        return image::decode_base64_image(b64);
    }
    if let Some(b) = body.filter(|b| !b.is_empty()) {
        return image::load_from_bytes(b);
    }
    Err(crate::utils::image::ImageError::Other("image is required".to_string()))
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

// ----- resize -----

async fn get_resize(
    web::Query(params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    core_resize(&params, None, false)
}

async fn post_resize(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_resize(&params, Some(&body), false)
}

async fn get_resize_base64(
    web::Query(params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    core_resize(&params, None, true)
}

async fn post_resize_base64(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_resize(&params, Some(&body), true)
}

fn core_resize(params: &HashMap<String, String>, body: Option<&[u8]>, as_base64: bool) -> impl Responder {
    let width = image::parse_u32(params, "width", 0);
    let height = image::parse_u32(params, "height", 0);
    if width == 0 || height == 0 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "width and height are required" }));
    }
    let mode = image::parse_str(params, "mode", "fit");
    let img = match load_input(params, body) {
        Ok(i) => i,
        Err(e) => return image::error_response(&e),
    };
    let result = apply_resize(&img, width, height, &mode);
    if as_base64 { respond_base64(result) } else { respond_png(result) }
}

// ----- rotate -----

async fn get_rotate(
    web::Query(params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    core_rotate(&params, None, false)
}

async fn post_rotate(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_rotate(&params, Some(&body), false)
}

async fn get_rotate_base64(
    web::Query(params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    core_rotate(&params, None, true)
}

async fn post_rotate_base64(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_rotate(&params, Some(&body), true)
}

fn core_rotate(params: &HashMap<String, String>, body: Option<&[u8]>, as_base64: bool) -> impl Responder {
    let deg = image::parse_u32(params, "deg", 90);
    let img = match load_input(params, body) {
        Ok(i) => i,
        Err(e) => return image::error_response(&e),
    };
    let result = apply_rotate(&img, deg);
    if as_base64 { respond_base64(result) } else { respond_png(result) }
}

// ----- flip -----

async fn get_flip(
    web::Query(params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    core_flip(&params, None, false)
}

async fn post_flip(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_flip(&params, Some(&body), false)
}

async fn get_flip_base64(
    web::Query(params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    core_flip(&params, None, true)
}

async fn post_flip_base64(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_flip(&params, Some(&body), true)
}

fn core_flip(params: &HashMap<String, String>, body: Option<&[u8]>, as_base64: bool) -> impl Responder {
    let direction = image::parse_str(params, "direction", "horizontal");
    let img = match load_input(params, body) {
        Ok(i) => i,
        Err(e) => return image::error_response(&e),
    };
    let result = apply_flip(&img, &direction);
    if as_base64 { respond_base64(result) } else { respond_png(result) }
}

// ----- grayscale -----

async fn get_grayscale(
    web::Query(params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    core_grayscale(&params, None, false)
}

async fn post_grayscale(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_grayscale(&params, Some(&body), false)
}

async fn get_grayscale_base64(
    web::Query(params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    core_grayscale(&params, None, true)
}

async fn post_grayscale_base64(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_grayscale(&params, Some(&body), true)
}

fn core_grayscale(params: &HashMap<String, String>, body: Option<&[u8]>, as_base64: bool) -> impl Responder {
    let img = match load_input(params, body) {
        Ok(i) => i,
        Err(e) => return image::error_response(&e),
    };
    let result = apply_grayscale(&img);
    if as_base64 { respond_base64(result) } else { respond_png(result) }
}

// ----- blur -----

async fn get_blur(
    web::Query(params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    core_blur(&params, None, false)
}

async fn post_blur(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_blur(&params, Some(&body), false)
}

async fn get_blur_base64(
    web::Query(params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    core_blur(&params, None, true)
}

async fn post_blur_base64(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_blur(&params, Some(&body), true)
}

fn core_blur(params: &HashMap<String, String>, body: Option<&[u8]>, as_base64: bool) -> impl Responder {
    let sigma = image::parse_f32(params, "sigma", 2.0);
    let img = match load_input(params, body) {
        Ok(i) => i,
        Err(e) => return image::error_response(&e),
    };
    let result = apply_blur(&img, sigma);
    if as_base64 { respond_base64(result) } else { respond_png(result) }
}
