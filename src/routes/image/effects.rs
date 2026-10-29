use actix_web::{web, HttpResponse, Responder};
use actix_multipart::Multipart;
use futures_util::stream::StreamExt as _;
use image::{DynamicImage, GenericImageView, ImageBuffer, Luma, Rgba};
use crate::utils::image::{self as utils_image, merge_json_body};
use imageproc::filter;
use imageproc::gradients;
use serde::Deserialize;
use std::collections::HashMap;

const DEFAULT_WIDTH: u32 = 512;
const DEFAULT_HEIGHT: u32 = 512;

#[derive(Deserialize, Clone)]
pub struct EffectPayload {
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default = "default_effect")]
    pub effect: String,
    #[serde(default = "default_sigma")]
    pub sigma: f32,
    #[serde(default = "default_amount")]
    pub amount: f32,
    #[serde(default = "default_x")]
    pub x: u32,
    #[serde(default = "default_y")]
    pub y: u32,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default = "default_factor")]
    pub factor: f32,
    #[serde(default = "default_threshold")]
    pub threshold: u8,
    #[serde(default = "default_angle")]
    pub angle: f32,
    #[serde(default = "default_radius")]
    pub radius: u32,
    #[serde(default = "default_strength")]
    pub strength: f32,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
}

fn default_effect() -> String {
    "blur".to_string()
}
fn default_sigma() -> f32 {
    2.0
}
fn default_amount() -> f32 {
    1.0
}
fn default_x() -> u32 {
    0
}
fn default_y() -> u32 {
    0
}
fn default_width() -> u32 {
    DEFAULT_WIDTH
}
fn default_height() -> u32 {
    DEFAULT_HEIGHT
}
fn default_factor() -> f32 {
    1.0
}
fn default_threshold() -> u8 {
    128
}
fn default_angle() -> f32 {
    0.0
}
fn default_radius() -> u32 {
    8
}
fn default_strength() -> f32 {
    1.0
}
fn default_opacity() -> f32 {
    0.5
}

impl Default for EffectPayload {
    fn default() -> Self {
        Self {
            image: None,
            url: None,
            effect: default_effect(),
            sigma: default_sigma(),
            amount: default_amount(),
            x: default_x(),
            y: default_y(),
            width: default_width(),
            height: default_height(),
            color: None,
            factor: default_factor(),
            threshold: default_threshold(),
            angle: default_angle(),
            radius: default_radius(),
            strength: default_strength(),
            opacity: default_opacity(),
        }
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.route("/image/effects", web::get().to(get_effects));
    cfg.route("/image/effects", web::post().to(post_effects));
    cfg.route(
        "/image/effects_base64",
        web::get().to(get_effects_base64),
    );
    cfg.route(
        "/image/effects_base64",
        web::post().to(post_effects_base64),
    );
    cfg.route(
        "/image/effects_raw",
        web::get().to(get_effects_raw),
    );
    cfg.route(
        "/image/effects_raw",
        web::post().to(post_effects_raw),
    );
    cfg.route(
        "/image/effects_multipart",
        web::get().to(get_effects_multipart),
    );
    cfg.route(
        "/image/effects_multipart",
        web::post().to(post_effects_multipart),
    );
    cfg.route("/image/effects_get", web::get().to(get_effects_get));
    cfg.route("/image/effects_get", web::post().to(post_effects_get));
    // legacy individual endpoints (dual method)
    cfg.route("/image/effects/sepia", web::get().to(get_sepia));
    cfg.route("/image/effects/sepia", web::post().to(post_sepia));
    cfg.route("/image/effects/sepia_base64", web::get().to(get_sepia_base64));
    cfg.route(
        "/image/effects/sepia_base64",
        web::post().to(post_sepia_base64),
    );
    cfg.route("/image/effects/invert", web::get().to(get_invert));
    cfg.route("/image/effects/invert", web::post().to(post_invert));
    cfg.route("/image/effects/invert_base64", web::get().to(get_invert_base64));
    cfg.route(
        "/image/effects/invert_base64",
        web::post().to(post_invert_base64),
    );
    cfg.route("/image/effects/edge", web::get().to(get_edge));
    cfg.route("/image/effects/edge", web::post().to(post_edge));
    cfg.route("/image/effects/edge_base64", web::get().to(get_edge_base64));
    cfg.route(
        "/image/effects/edge_base64",
        web::post().to(post_edge_base64),
    );
}

// ---------- shared core ----------

fn load_input_image_from_params(
    params: &HashMap<String, String>,
    body: Option<&[u8]>,
) -> Result<DynamicImage, crate::utils::image::ImageError> {
    if let Some(b64) = params.get("image").filter(|s| !s.is_empty()) {
        return crate::utils::image::decode_base64_image(b64);
    }
    if let Some(b) = body.filter(|b| !b.is_empty()) {
        return crate::utils::image::load_from_bytes(b);
    }
    if let Some(url) = params.get("url").filter(|s| !s.is_empty()) {
        let fetched = reqwest::blocking::get(url)
            .map_err(|e| crate::utils::image::ImageError::Other(format!("fetch failed: {e}")))?
            .bytes()
            .map_err(|e| crate::utils::image::ImageError::Other(format!("read failed: {e}")))?;
        return utils_image::load_from_bytes(&fetched);
    }
    Err(crate::utils::image::ImageError::InvalidImageFormat(
        "image or url is required".to_string(),
    ))
}

fn respond_png(result: DynamicImage) -> HttpResponse {
    match utils_image::encode_png_bytes(&result) {
        Ok(bytes) => HttpResponse::Ok().content_type("image/png").body(bytes),
        Err(e) => utils_image::error_response(&e),
    }
}

fn respond_base64(result: DynamicImage) -> HttpResponse {
    match utils_image::encode_base64_png(&result) {
        Ok(b64) => HttpResponse::Ok().json(serde_json::json!({ "image_base64": b64 })),
        Err(e) => utils_image::error_response(&e),
    }
}

fn apply_effect(img: &mut DynamicImage, p: &EffectPayload) -> DynamicImage {
    match p.effect.as_str() {
        "blur" => apply_blur(img, p.sigma),
        "brightness" => apply_brightness(img, p.amount),
        "contrast" => apply_contrast(img, p.amount),
        "sharpen" => apply_sharpen(img, p.amount),
        "grayscale" => apply_grayscale(img),
        "invert" => apply_invert(img),
        "normalize" => apply_normalize(img),
        "sepia" => apply_sepia(img),
        "flip" => apply_flip(img),
        "resize" => apply_resize(img, p.width, p.height),
        "crop" => apply_crop(img, p.x, p.y, p.width, p.height),
        "scale" => apply_resize(img, p.width, p.height),
        "contain" => apply_contain(img, p.width, p.height),
        "cover" => apply_cover(img, p.width, p.height),
        "opacity" => apply_opacity(img, p.opacity),
        "fade" => apply_fade(img, p.amount),
        "saturation" => apply_saturation(img, p.amount),
        "hue" => apply_hue(img, p.angle),
        "correction" => apply_correction(img),
        "threshold" => apply_threshold(img, p.threshold),
        "tint" => apply_tint(img, p.color.as_deref()),
        "adjustment" => apply_adjustment(img, p.amount),
        "median" => apply_median(img, p.radius),
        "pixelate" => apply_pixelate(img, p.radius),
        "convolute" => apply_convolute(img, p.factor),
        "gaussian" => apply_blur(img, p.sigma),
        "posturize" => apply_posturize(img, p.amount as u8),
        "dither" => apply_dither(img),
        "circle" => apply_circle(img),
        "fisheye" => apply_fisheye(img, p.amount),
        "drop_shadow" => apply_drop_shadow(img, p.x, p.y, p.radius, p.opacity),
        "displace" => apply_displace(img, p.amount),
        "mirror" => apply_mirror(img),
        "glitch" => apply_glitch(img, p.amount),
        "vignette" => apply_vignette(img, p.amount),
        "vintage" => apply_vintage(img),
        "emboss" => apply_emboss(img),
        "oil_painting" => apply_oil_painting(img, p.radius),
        "duo_tone" => apply_duo_tone(img, p.color.as_deref()),
        "chromatic" => apply_chromatic(img, p.amount),
        "halftone" => apply_halftone(img, p.radius),
        "ascii_art" => apply_ascii_art(img),
        "scanlines" => apply_scanlines(img, p.amount),
        "motion_blur" => apply_motion_blur(img, p.angle, p.radius),
        "kaleidoscope" => apply_kaleidoscope(img),
        "pixel_sort" => apply_pixel_sort(img),
        "wave_distortion" => apply_wave_distortion(img, p.amount),
        "swirl" => apply_swirl(img, p.amount),
        "bulge" => apply_bulge(img, p.amount),
        "noise" => apply_noise(img, p.amount),
        "denoise" => apply_denoise(img, p.sigma),
        "cartoon" => apply_cartoon(img),
        "pencil_sketch" => apply_pencil_sketch(img),
        "hdr" => apply_hdr(img),
        "bokeh" => apply_bokeh(img, p.radius),
        "thermal" => apply_thermal(img),
        "xray" => apply_xray(img),
        "infrared" => apply_infrared(img),
        "auto_enhance" => apply_auto_enhance(img),
        "remove_background" => apply_remove_background(img),
        "vhs_effect" => apply_vhs_effect(img),
        "neon_glow" => apply_neon_glow(img, p.amount),
        "polaroid_frame" => apply_polaroid_frame(img),
        "watermark" => apply_watermark(img, p.color.as_deref(), p.opacity),
        _ => apply_blur(img, p.sigma),
    }
}

// payload from query+json body
fn payload_from(params: &HashMap<String, String>, body: Option<&[u8]>) -> EffectPayload {
    let mut p = EffectPayload::default();
    for (k, v) in params {
        match k.as_str() {
            "effect" => p.effect = v.clone(),
            "url" => p.url = Some(v.clone()),
            "image" => p.image = Some(v.clone()),
            "sigma" => p.sigma = v.parse().unwrap_or(2.0),
            "amount" => p.amount = v.parse().unwrap_or(1.0),
            "x" => p.x = v.parse().unwrap_or(0),
            "y" => p.y = v.parse().unwrap_or(0),
            "width" => p.width = v.parse().unwrap_or(DEFAULT_WIDTH),
            "height" => p.height = v.parse().unwrap_or(DEFAULT_HEIGHT),
            "color" => p.color = Some(v.clone()),
            "factor" => p.factor = v.parse().unwrap_or(1.0),
            "threshold" => p.threshold = v.parse().unwrap_or(128),
            "angle" => p.angle = v.parse().unwrap_or(0.0),
            "radius" => p.radius = v.parse().unwrap_or(8),
            "strength" => p.strength = v.parse().unwrap_or(1.0),
            "opacity" => p.opacity = v.parse().unwrap_or(0.5),
            _ => {}
        }
    }
    if let Some(b) = body {
        if let Ok(val) = serde_json::from_slice::<serde_json::Value>(b) {
            if let serde_json::Value::Object(map) = val {
                for (k, v) in map {
                    let ks = k.as_str();
                    let vs = v.to_string();
                    match ks {
                        "effect" => p.effect = vs.clone(),
                        "url" => p.url = Some(vs.clone()),
                        "image" => p.image = Some(vs.clone()),
                        "sigma" => p.sigma = vs.parse().unwrap_or(2.0),
                        "amount" => p.amount = vs.parse().unwrap_or(1.0),
                        "x" => p.x = vs.parse().unwrap_or(0),
                        "y" => p.y = vs.parse().unwrap_or(0),
                        "width" => p.width = vs.parse().unwrap_or(DEFAULT_WIDTH),
                        "height" => p.height = vs.parse().unwrap_or(DEFAULT_HEIGHT),
                        "color" => p.color = Some(vs.clone()),
                        "factor" => p.factor = vs.parse().unwrap_or(1.0),
                        "threshold" => p.threshold = vs.parse().unwrap_or(128),
                        "angle" => p.angle = vs.parse().unwrap_or(0.0),
                        "radius" => p.radius = vs.parse().unwrap_or(8),
                        "strength" => p.strength = vs.parse().unwrap_or(1.0),
                        "opacity" => p.opacity = vs.parse().unwrap_or(0.5),
                        _ => {}
                    }
                }
            }
        }
    }
    p
}

// ----- main effects endpoint -----

async fn get_effects(
    web::Query(params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    core_effects(&params, None, false)
}

async fn post_effects(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_effects(&params, Some(&body), false)
}

async fn get_effects_base64(
    web::Query(params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    core_effects(&params, None, true)
}

async fn post_effects_base64(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_effects(&params, Some(&body), true)
}

async fn get_effects_raw(
    web::Query(params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    core_effects(&params, None, false)
}

async fn post_effects_raw(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_effects(&params, Some(&body), false)
}

fn core_effects(params: &HashMap<String, String>, body: Option<&[u8]>, as_base64: bool) -> impl Responder {
    let p = payload_from(params, body);
    let img = match load_input_image_from_params(params, body) {
        Ok(i) => i,
        Err(e) => return utils_image::error_response(&e),
    };
    let mut img = img;
    let result = apply_effect(&mut img, &p);
    if as_base64 { respond_base64(result) } else { respond_png(result) }
}

// ----- multipart -----

async fn get_effects_multipart() -> impl Responder {
    HttpResponse::MethodNotAllowed().json(serde_json::json!({ "error": "use POST with multipart form" }))
}

async fn post_effects_multipart(mut payload: Multipart) -> impl Responder {
    let mut img_bytes: Option<Vec<u8>> = None;
    let mut params: HashMap<String, String> = HashMap::new();
    while let Some(item) = payload.next().await {
        if let Ok(mut field) = item {
            let name = field.name().unwrap_or("").to_string();
            let bytes: Vec<u8> = match field.next().await {
                Some(Ok(b)) => b.to_vec(),
                _ => Vec::new(),
            };
            if name == "image" && !bytes.is_empty() {
                img_bytes = Some(bytes);
            } else if !bytes.is_empty() {
                if let Ok(s) = std::str::from_utf8(&bytes) {
                    params.insert(name, s.to_string());
                }
            }
        }
    }
    let effect = params.remove("effect").unwrap_or_else(|| "blur".to_string());
    let sigma: f32 = params
        .get("sigma")
        .and_then(|v| v.parse().ok())
        .unwrap_or(2.0);
    let img = match img_bytes {
        Some(b) => match image::load_from_memory(&b) {
            Ok(i) => i,
            Err(e) => return utils_image::error_response(&crate::utils::image::ImageError::InvalidImageFormat(e.to_string())),
        },
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({ "error": "image field is required" }));
        }
    };
    let mut p = EffectPayload {
        effect,
        sigma,
        ..Default::default()
    };
    for (k, v) in params {
        match k.as_str() {
            "amount" => p.amount = v.parse().unwrap_or(1.0),
            "x" => p.x = v.parse().unwrap_or(0),
            "y" => p.y = v.parse().unwrap_or(0),
            "width" => p.width = v.parse().unwrap_or(DEFAULT_WIDTH),
            "height" => p.height = v.parse().unwrap_or(DEFAULT_HEIGHT),
            "color" => p.color = Some(v),
            "factor" => p.factor = v.parse().unwrap_or(1.0),
            "threshold" => p.threshold = v.parse().unwrap_or(128),
            "angle" => p.angle = v.parse().unwrap_or(0.0),
            "radius" => p.radius = v.parse().unwrap_or(8),
            "strength" => p.strength = v.parse().unwrap_or(1.0),
            "opacity" => p.opacity = v.parse().unwrap_or(0.5),
            _ => {}
        }
    }
    let mut img = img;
    let result = apply_effect(&mut img, &p);
    respond_png(result)
}

// ----- effects_get (url fetch) -----

async fn get_effects_get(
    web::Query(params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    core_effects_get(&params, None)
}

async fn post_effects_get(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_effects_get(&params, None)
}

fn core_effects_get(params: &HashMap<String, String>, _body: Option<&[u8]>) -> impl Responder {
    let url = match params.get("url").cloned() {
        Some(u) => u,
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({ "error": "url query param is required" }));
        }
    };
    let p = payload_from(params, None);
    let fetched = match reqwest::blocking::get(&url) {
        Ok(r) => match r.bytes() {
            Ok(b) => b.to_vec(),
            Err(e) => {
                return HttpResponse::BadGateway()
                    .json(serde_json::json!({ "error": format!("fetch failed: {e}") }));
            }
        },
        Err(e) => {
            return HttpResponse::BadGateway()
                .json(serde_json::json!({ "error": format!("fetch failed: {e}") }));
        }
    };
    let img = match image::load_from_memory(&fetched) {
        Ok(i) => i,
        Err(e) => return utils_image::error_response(&crate::utils::image::ImageError::InvalidImageFormat(e.to_string())),
    };
    let mut img = img;
    let result = apply_effect(&mut img, &p);
    respond_png(result)
}

// ----- legacy individual endpoints -----

async fn get_sepia(web::Query(params): web::Query<HashMap<String, String>>) -> impl Responder {
    core_legacy(&params, None, apply_sepia, true)
}
async fn post_sepia(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_legacy(&params, Some(&body), apply_sepia, true)
}
async fn get_sepia_base64(
    web::Query(params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    core_legacy(&params, None, apply_sepia, false)
}
async fn post_sepia_base64(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_legacy(&params, Some(&body), apply_sepia, false)
}

async fn get_invert(web::Query(params): web::Query<HashMap<String, String>>) -> impl Responder {
    core_legacy(&params, None, apply_invert, true)
}
async fn post_invert(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_legacy(&params, Some(&body), apply_invert, true)
}
async fn get_invert_base64(
    web::Query(params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    core_legacy(&params, None, apply_invert, false)
}
async fn post_invert_base64(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_legacy(&params, Some(&body), apply_invert, false)
}

async fn get_edge(web::Query(params): web::Query<HashMap<String, String>>) -> impl Responder {
    core_legacy(&params, None, apply_edge_effect, true)
}
async fn post_edge(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_legacy(&params, Some(&body), apply_edge_effect, true)
}
async fn get_edge_base64(
    web::Query(params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    core_legacy(&params, None, apply_edge_effect, false)
}
async fn post_edge_base64(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_legacy(&params, Some(&body), apply_edge_effect, false)
}

fn core_legacy(
    params: &HashMap<String, String>,
    body: Option<&[u8]>,
    f: fn(&DynamicImage) -> DynamicImage,
    raw: bool,
) -> impl Responder {
    let img = match load_input_image_from_params(params, body) {
        Ok(i) => i,
        Err(e) => return utils_image::error_response(&e),
    };
    let result = f(&img);
    if raw { respond_png(result) } else { respond_base64(result) }
}

// ---------- effect implementations ----------

fn apply_blur(img: &DynamicImage, sigma: f32) -> DynamicImage {
    if sigma <= 0.0 {
        return img.clone();
    }
    let rgba = img.to_rgba8();
    DynamicImage::ImageRgba8(filter::gaussian_blur_f32(&rgba, sigma))
}

fn apply_brightness(img: &DynamicImage, amount: f32) -> DynamicImage {
    let mut out = img.clone();
    image::imageops::brighten(&mut out, (amount * 50.0) as i32);
    out
}

fn apply_contrast(img: &DynamicImage, amount: f32) -> DynamicImage {
    let mut out = img.clone();
    image::imageops::contrast(&mut out, amount);
    out
}

fn apply_sharpen(img: &DynamicImage, amount: f32) -> DynamicImage {
    let rgba = img.to_rgba8();
    let radius = if amount > 1.0 { 3.0 } else { 1.0 };
    let sharp = image::imageops::unsharpen(&rgba, radius, 10);
    DynamicImage::ImageRgba8(sharp)
}

fn apply_grayscale(img: &DynamicImage) -> DynamicImage {
    DynamicImage::ImageLuma8(image::imageops::grayscale(img))
}

fn apply_invert(img: &DynamicImage) -> DynamicImage {
    let mut out = img.clone();
    image::imageops::invert(&mut out);
    out
}

fn apply_normalize(img: &DynamicImage) -> DynamicImage {
    let mut out = img.to_rgba8();
    let mut min = [u8::MAX, u8::MAX, u8::MAX];
    let mut max = [0u8, 0u8, 0u8];
    for y in 0..out.height() {
        for x in 0..out.width() {
            let p = out.get_pixel(x, y).0;
            for c in 0..3usize {
                if p[c] < min[c] {
                    min[c] = p[c];
                }
                if p[c] > max[c] {
                    max[c] = p[c];
                }
            }
        }
    }
    for y in 0..out.height() {
        for x in 0..out.width() {
            let p = out.get_pixel_mut(x, y);
            for c in 0..3usize {
                let range = max[c] as i32 - min[c] as i32;
                p.0[c] = if range > 0 {
                    ((p.0[c] as i32 - min[c] as i32) * 255 / range).clamp(0, 255) as u8
                } else {
                    p.0[c]
                };
            }
        }
    }
    DynamicImage::ImageRgba8(out)
}

fn apply_sepia(img: &DynamicImage) -> DynamicImage {
    let src = img.to_rgba8();
    let mut out = src.clone();
    for y in 0..src.height() {
        for x in 0..src.width() {
            let p = src.get_pixel(x, y);
            let r = p.0[0] as f32;
            let g = p.0[1] as f32;
            let b = p.0[2] as f32;
            let nr = (0.393_f32 * r + 0.769 * g + 0.189 * b).min(255.0_f32) as u8;
            let ng = (0.349_f32 * r + 0.686 * g + 0.168 * b).min(255.0_f32) as u8;
            let nb = (0.272_f32 * r + 0.534 * g + 0.131 * b).min(255.0_f32) as u8;
            out.put_pixel(x, y, Rgba([nr, ng, nb, p.0[3]]));
        }
    }
    DynamicImage::ImageRgba8(out)
}

fn apply_flip(img: &DynamicImage) -> DynamicImage {
    let rgba = img.to_rgba8();
    DynamicImage::ImageRgba8(image::imageops::flip_horizontal(&rgba))
}

fn apply_resize(img: &DynamicImage, width: u32, height: u32) -> DynamicImage {
    let w = width.max(1);
    let h = height.max(1);
    DynamicImage::ImageRgba8(image::imageops::resize(
        img,
        w,
        h,
        image::imageops::FilterType::Triangle,
    ))
}

fn apply_crop(img: &DynamicImage, x: u32, y: u32, width: u32, height: u32) -> DynamicImage {
    let w = width.max(1);
    let h = height.max(1);
    let x = x.min(img.width().saturating_sub(1));
    let y = y.min(img.height().saturating_sub(1));
    let crop_w = w.min(img.width().saturating_sub(x));
    let crop_h = h.min(img.height().saturating_sub(y));
    let crop = image::imageops::crop_imm(img, x, y, crop_w, crop_h);
    DynamicImage::ImageRgba8(crop.to_image())
}

fn apply_contain(img: &DynamicImage, width: u32, height: u32) -> DynamicImage {
    DynamicImage::ImageRgba8(image::imageops::resize(
        img,
        width.max(1),
        height.max(1),
        image::imageops::FilterType::Triangle,
    ))
}

fn apply_cover(img: &DynamicImage, width: u32, height: u32) -> DynamicImage {
    DynamicImage::ImageRgba8(image::imageops::resize(
        img,
        width.max(1),
        height.max(1),
        image::imageops::FilterType::Triangle,
    ))
}

fn apply_opacity(img: &DynamicImage, opacity: f32) -> DynamicImage {
    let mut out = img.to_rgba8();
    let a = (opacity.clamp(0.0, 1.0) * 255.0) as u8;
    for y in 0..out.height() {
        for x in 0..out.width() {
            let p = out.get_pixel_mut(x, y);
            p.0[3] = ((p.0[3] as u32 * a as u32) / 255) as u8;
        }
    }
    DynamicImage::ImageRgba8(out)
}

fn apply_fade(img: &DynamicImage, amount: f32) -> DynamicImage {
    let mut out = img.clone();
    image::imageops::brighten(&mut out, -(amount * 50.0) as i32);
    out
}

fn apply_saturation(img: &DynamicImage, amount: f32) -> DynamicImage {
    let gray = image::imageops::grayscale(img);
    let gray_rgba = DynamicImage::ImageLuma8(gray).to_rgba8();
    let mut out = img.to_rgba8();
    let a = amount.clamp(0.0, 2.0);
    for y in 0..out.height() {
        for x in 0..out.width() {
            let p = out.get_pixel(x, y);
            let g = gray_rgba.get_pixel(x, y).0[0];
            let pc = [p.0[0], p.0[1], p.0[2]];
            for c in 0..3usize {
                let blended = (pc[c] as f32 * a + g as f32 * (1.0 - a)).clamp(0.0, 255.0) as u8;
                let mut p = *out.get_pixel(x, y);
                p.0[c] = blended;
                out.put_pixel(x, y, p);
            }
        }
    }
    DynamicImage::ImageRgba8(out)
}

fn apply_hue(img: &DynamicImage, angle: f32) -> DynamicImage {
    let mut out = img.clone();
    image::imageops::huerotate(&mut out, angle.to_radians() as i32);
    out
}

fn apply_correction(img: &DynamicImage) -> DynamicImage {
    apply_normalize(img)
}

fn apply_threshold(img: &DynamicImage, threshold: u8) -> DynamicImage {
    let gray = image::imageops::grayscale(img);
    let mut out = ImageBuffer::from_pixel(img.width(), img.height(), Luma([0]));
    for y in 0..gray.height() {
        for x in 0..gray.width() {
            let v = gray.get_pixel(x, y).0[0];
            out.put_pixel(x, y, Luma([if v >= threshold { 255 } else { 0 }]));
        }
    }
    DynamicImage::ImageLuma8(out)
}

fn apply_tint(img: &DynamicImage, color: Option<&str>) -> DynamicImage {
    let (r, g, b) = color.and_then(|c| parse_hex_color(c)).unwrap_or((100u8, 149u8, 237u8));
    let mut out = img.to_rgba8();
    let a = 0.5f32;
    for y in 0..out.height() {
        for x in 0..out.width() {
            let p = out.get_pixel(x, y);
            let pc = [p.0[0], p.0[1], p.0[2]];
            for c in 0..3usize {
                let blended = (pc[c] as f32 * (1.0 - a) + (c as u32 as f32 * 0.0).clamp(0.0, 255.0)) as u8;
                let _ = blended;
            }
            let pc2 = [(r as f32 * a + p.0[0] as f32 * (1.0 - a)) as u8,
                       (g as f32 * a + p.0[1] as f32 * (1.0 - a)) as u8,
                       (b as f32 * a + p.0[2] as f32 * (1.0 - a)) as u8];
            out.put_pixel(x, y, Rgba([pc2[0], pc2[1], pc2[2], p.0[3]]));
        }
    }
    DynamicImage::ImageRgba8(out)
}

fn apply_adjustment(img: &DynamicImage, amount: f32) -> DynamicImage {
    let out = apply_contrast(img, amount);
    apply_brightness(&out, amount)
}

fn apply_median(img: &DynamicImage, radius: u32) -> DynamicImage {
    let rgba = img.to_rgba8();
    DynamicImage::ImageRgba8(imageproc::filter::median_filter(&rgba, radius.max(1), radius.max(1)))
}

fn apply_pixelate(img: &DynamicImage, radius: u32) -> DynamicImage {
    let r = radius.max(2).min(32);
    let small = image::imageops::resize(
        img,
        img.width().max(1) / r,
        img.height().max(1) / r,
        image::imageops::FilterType::Nearest,
    );
    DynamicImage::ImageRgba8(image::imageops::resize(
        &small,
        img.width(),
        img.height(),
        image::imageops::FilterType::Nearest,
    ))
}

fn apply_convolute(img: &DynamicImage, _factor: f32) -> DynamicImage {
    apply_emboss(img)
}

fn apply_posturize(img: &DynamicImage, bits: u8) -> DynamicImage {
    let bits = bits.clamp(1, 8);
    let shift = 8 - bits;
    let mut out = img.to_rgba8();
    for y in 0..out.height() {
        for x in 0..out.width() {
            let p = out.get_pixel_mut(x, y);
            p.0[0] = (p.0[0] >> shift << shift) | (1u8 << shift).saturating_sub(1);
            p.0[1] = (p.0[1] >> shift << shift) | (1u8 << shift).saturating_sub(1);
            p.0[2] = (p.0[2] >> shift << shift) | (1u8 << shift).saturating_sub(1);
        }
    }
    DynamicImage::ImageRgba8(out)
}

fn apply_dither(img: &DynamicImage) -> DynamicImage {
    let gray = image::imageops::grayscale(img);
    let mut out = ImageBuffer::from_pixel(gray.width(), gray.height(), Luma([0]));
    for y in 0..gray.height() {
        for x in 0..gray.width() {
            let v = gray.get_pixel(x, y).0[0];
            let new_v = if v >= 128 { 255 } else { 0 };
            out.put_pixel(x, y, Luma([new_v]));
        }
    }
    DynamicImage::ImageLuma8(out)
}

fn apply_circle(img: &DynamicImage) -> DynamicImage {
    let mut out = img.to_rgba8();
    let cx = out.width() / 2;
    let cy = out.height() / 2;
    let r = cx.min(cy);
    for y in 0..out.height() {
        for x in 0..out.width() {
            let dx = x as i32 - cx as i32;
            let dy = y as i32 - cy as i32;
            if (dx * dx + dy * dy) > (r as i32 * r as i32) {
                out.get_pixel_mut(x, y).0[3] = 0;
            }
        }
    }
    DynamicImage::ImageRgba8(out)
}

fn apply_fisheye(img: &DynamicImage, _amount: f32) -> DynamicImage {
    img.clone()
}

fn apply_drop_shadow(img: &DynamicImage, _x: u32, _y: u32, radius: u32, _opacity: f32) -> DynamicImage {
    apply_blur(img, radius as f32 * 2.0)
}

fn apply_displace(img: &DynamicImage, _amount: f32) -> DynamicImage {
    img.clone()
}

fn apply_mirror(img: &DynamicImage) -> DynamicImage {
    let rgba = img.to_rgba8();
    DynamicImage::ImageRgba8(image::imageops::flip_vertical(&rgba))
}

fn apply_glitch(img: &DynamicImage, amount: f32) -> DynamicImage {
    let mut out = img.to_rgba8();
    let shift = (amount * 20.0) as u32;
    for y in 0..out.height() {
        if y % 4 == 0 {
            for x in shift..out.width() {
                let p = out.get_pixel(x - shift, y);
                out.get_pixel_mut(x, y).0[0] = p.0[0];
            }
        }
    }
    DynamicImage::ImageRgba8(out)
}

fn apply_vignette(img: &DynamicImage, amount: f32) -> DynamicImage {
    let mut out = img.to_rgba8();
    let cx = out.width() as f32 / 2.0;
    let cy = out.height() as f32 / 2.0;
    let max_d = (cx * cx + cy * cy).sqrt();
    let a = amount.clamp(0.0, 1.0);
    for y in 0..out.height() {
        for x in 0..out.width() {
            let d = ((x as f32 - cx) * (x as f32 - cx) + (y as f32 - cy) * (y as f32 - cy)).sqrt();
            let v = (1.0 - (d / max_d) * a).clamp(0.0, 1.0);
            let p = out.get_pixel_mut(x, y);
            p.0[0] = (p.0[0] as f32 * v) as u8;
            p.0[1] = (p.0[1] as f32 * v) as u8;
            p.0[2] = (p.0[2] as f32 * v) as u8;
        }
    }
    DynamicImage::ImageRgba8(out)
}

fn apply_vintage(img: &DynamicImage) -> DynamicImage {
    let out = apply_sepia(img);
    apply_contrast(&out, 1.2)
}

fn apply_emboss(img: &DynamicImage) -> DynamicImage {
    let gray = image::imageops::grayscale(img);
    let mut out = ImageBuffer::from_pixel(gray.width(), gray.height(), Luma([128]));
    let kernel = [[-2i16, -1, 0], [-1, 1, 1], [0, 1, 2]];
    for y in 1..gray.height() - 1 {
        for x in 1..gray.width() - 1 {
            let mut sum = 0i32;
            for ky in 0..3 {
                for kx in 0..3 {
                    let p = gray.get_pixel(x - 1 + kx, y - 1 + ky).0[0] as i32;
                    sum += p * kernel[ky as usize][kx as usize] as i32;
                }
            }
            out.put_pixel(x, y, Luma([sum.clamp(0, 255) as u8]));
        }
    }
    DynamicImage::ImageLuma8(out)
}

fn apply_oil_painting(img: &DynamicImage, _radius: u32) -> DynamicImage {
    img.clone()
}

fn apply_duo_tone(img: &DynamicImage, color: Option<&str>) -> DynamicImage {
    let (r2, g2, b2) = color.and_then(|c| parse_hex_color(c)).unwrap_or((0u8, 128u8, 255u8));
    let gray = image::imageops::grayscale(img);
    let mut out = ImageBuffer::from_pixel(gray.width(), gray.height(), Rgba([0, 0, 0, 255]));
    for y in 0..gray.height() {
        for x in 0..gray.width() {
            let v = gray.get_pixel(x, y).0[0] as f32 / 255.0;
            out.put_pixel(
                x,
                y,
                Rgba([
                    (r2 as f32 * v) as u8,
                    (g2 as f32 * v) as u8,
                    (b2 as f32 * v) as u8,
                    255,
                ]),
            );
        }
    }
    DynamicImage::ImageRgba8(out)
}

fn apply_chromatic(img: &DynamicImage, amount: f32) -> DynamicImage {
    let mut out = img.to_rgba8();
    let shift = (amount * 5.0) as u32;
    let w = out.width();
    let h = out.height();
    for y in 0..h {
        for x in 0..w {
            if x >= shift && x + shift < w {
                let p = out.get_pixel(x - shift, y);
                out.get_pixel_mut(x, y).0[0] = p.0[0];
            }
            if x + shift < w {
                let p = out.get_pixel(x + shift, y);
                out.get_pixel_mut(x, y).0[2] = p.0[2];
            }
        }
    }
    DynamicImage::ImageRgba8(out)
}

fn apply_halftone(img: &DynamicImage, radius: u32) -> DynamicImage {
    let r = radius.max(2);
    let gray = image::imageops::grayscale(img);
    let mut out = ImageBuffer::from_pixel(gray.width(), gray.height(), Luma([255]));
    for y in (0..gray.height()).step_by(r as usize) {
        for x in (0..gray.width()).step_by(r as usize) {
            let mut sum = 0u32;
            let mut cnt = 0u32;
            for dy in 0..r {
                for dx in 0..r {
                    if x + dx < gray.width() && y + dy < gray.height() {
                        sum += gray.get_pixel(x + dx, y + dy).0[0] as u32;
                        cnt += 1;
                    }
                }
            }
            let avg = if cnt > 0 { sum / cnt } else { 0 };
            let size = ((255 - avg) as u32 * r / 255).min(r);
            for dy in 0..size {
                for dx in 0..size {
                    if x + dx < gray.width() && y + dy < gray.height() {
                        out.put_pixel(x + dx, y + dy, Luma([0]));
                    }
                }
            }
        }
    }
    DynamicImage::ImageLuma8(out)
}

fn apply_ascii_art(img: &DynamicImage) -> DynamicImage {
    let gray = image::imageops::grayscale(img);
    let chars = " .:-=+*#%@";
    let mut out = ImageBuffer::from_pixel(gray.width(), gray.height(), Luma([255]));
    for y in 0..gray.height() {
        for x in 0..gray.width() {
            let v = gray.get_pixel(x, y).0[0] as usize;
            let idx = (v * (chars.len() - 1)) / 255;
            let ch = chars.as_bytes()[idx.min(chars.len() - 1)];
            if ch != b' ' {
                out.put_pixel(x, y, Luma([0]));
            }
        }
    }
    DynamicImage::ImageLuma8(out)
}

fn apply_scanlines(img: &DynamicImage, amount: f32) -> DynamicImage {
    let mut out = img.to_rgba8();
    let a = amount.clamp(0.0, 1.0);
    for y in 0..out.height() {
        if y % 2 == 0 {
            for x in 0..out.width() {
                let p = out.get_pixel(x, y);
                out.put_pixel(
                    x,
                    y,
                    Rgba([
                        (p.0[0] as f32 * (1.0 - a)) as u8,
                        (p.0[1] as f32 * (1.0 - a)) as u8,
                        (p.0[2] as f32 * (1.0 - a)) as u8,
                        p.0[3],
                    ]),
                );
            }
        }
    }
    DynamicImage::ImageRgba8(out)
}

fn apply_motion_blur(img: &DynamicImage, _angle: f32, radius: u32) -> DynamicImage {
    apply_blur(img, radius as f32)
}

fn apply_kaleidoscope(img: &DynamicImage) -> DynamicImage {
    let mut out = img.to_rgba8();
    let w = out.width();
    let h = out.height();
    for y in 0..h {
        for x in w / 2..w {
            let p = out.get_pixel(w as u32 - 1 - (x as u32).wrapping_sub(w as u32 / 2) % w as u32, y);
            out.put_pixel(x, y, Rgba([p.0[0], p.0[1], p.0[2], p.0[3]]));
        }
    }
    DynamicImage::ImageRgba8(out)
}

fn apply_pixel_sort(img: &DynamicImage) -> DynamicImage {
    let mut out = img.to_rgba8();
    for y in 0..out.height() {
        let mut row: Vec<(u32, u32, u32)> = Vec::new();
        for x in 0..out.width() {
            let p = out.get_pixel(x, y);
            row.push((p.0[0] as u32 + p.0[1] as u32 + p.0[2] as u32, x, y));
        }
        row.sort();
        for (i, (_, x, _)) in row.iter().enumerate() {
            let src = row[i].0 as u8;
            out.put_pixel(*x, y, Rgba([src, src, src, 255]));
        }
    }
    DynamicImage::ImageRgba8(out)
}

fn apply_wave_distortion(img: &DynamicImage, amount: f32) -> DynamicImage {
    let mut out = ImageBuffer::from_pixel(img.width(), img.height(), Rgba([0, 0, 0, 0]));
    let a = amount.clamp(0.0, 50.0);
    for y in 0..img.height() {
        for x in 0..img.width() {
            let sx = x as f32 + a * ((y as f32 / 20.0).sin());
            let sx = sx.clamp(0.0, img.width() as f32 - 1.0) as u32;
            let p = img.get_pixel(sx, y);
            out.put_pixel(x, y, Rgba([p.0[0], p.0[1], p.0[2], p.0[3]]));
        }
    }
    DynamicImage::ImageRgba8(out)
}

fn apply_swirl(img: &DynamicImage, amount: f32) -> DynamicImage {
    let mut out = ImageBuffer::from_pixel(img.width(), img.height(), Rgba([0, 0, 0, 0]));
    let cx = img.width() as f32 / 2.0;
    let cy = img.height() as f32 / 2.0;
    let a = amount.clamp(0.0, 2.0);
    for y in 0..img.height() {
        for x in 0..img.width() {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            let theta = a * (1.0 - dist / (cx.max(cy))).clamp(0.0, 1.0);
            let sx = (dx * theta.cos() - dy * theta.sin() + cx) as u32;
            let sy = (dx * theta.sin() + dy * theta.cos() + cy) as u32;
            if sx < img.width() && sy < img.height() {
                let p = img.get_pixel(sx, sy);
                out.put_pixel(x, y, Rgba([p.0[0], p.0[1], p.0[2], p.0[3]]));
            }
        }
    }
    DynamicImage::ImageRgba8(out)
}

fn apply_bulge(img: &DynamicImage, amount: f32) -> DynamicImage {
    let mut out = ImageBuffer::from_pixel(img.width(), img.height(), Rgba([0, 0, 0, 0]));
    let cx = img.width() as f32 / 2.0;
    let cy = img.height() as f32 / 2.0;
    let r = cx.min(cy);
    let a = amount.clamp(0.0, 1.0);
    for y in 0..img.height() {
        for x in 0..img.width() {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < r {
                let factor = 1.0 + a * (1.0 - dist / r);
                let sx = (cx + dx * factor) as u32;
                let sy = (cy + dy * factor) as u32;
                if sx < img.width() && sy < img.height() {
                    let p = img.get_pixel(sx, sy);
                    out.put_pixel(x, y, Rgba([p.0[0], p.0[1], p.0[2], p.0[3]]));
                }
            }
        }
    }
    DynamicImage::ImageRgba8(out)
}

fn apply_noise(img: &DynamicImage, amount: f32) -> DynamicImage {
    let mut out = img.to_rgba8();
    let a = (amount * 60.0) as i32;
    let mut seed: u32 = 12345;
    for y in 0..out.height() {
        for x in 0..out.width() {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let n = ((seed >> 16) % 256) as i32 - 128;
            let p = out.get_pixel_mut(x, y);
            let pc = [p.0[0], p.0[1], p.0[2]];
            for c in 0..3 {
                let v = (pc[c] as i32 + (n * a / 50)).clamp(0, 255) as u8;
                p.0[c] = v;
            }
        }
    }
    DynamicImage::ImageRgba8(out)
}

fn apply_denoise(img: &DynamicImage, sigma: f32) -> DynamicImage {
    apply_blur(img, sigma)
}

fn apply_cartoon(img: &DynamicImage) -> DynamicImage {
    let edges = apply_edge_effect(img);
    let mut quant = match apply_posterize_colors(img, 6) {
        DynamicImage::ImageRgba8(q) => q,
        other => other.to_rgba8(),
    };
    let e = edges.to_rgba8();
    for y in 0..quant.height() {
        for x in 0..quant.width() {
            let ep = e.get_pixel(x, y);
            if ep.0[0] < 100 {
                quant.put_pixel(x, y, Rgba([0, 0, 0, ep.0[3]]));
            }
        }
    }
    DynamicImage::ImageRgba8(quant)
}

fn apply_pencil_sketch(img: &DynamicImage) -> DynamicImage {
    let gray = image::imageops::grayscale(img);
    let mut inv = gray.clone();
    image::imageops::invert(&mut inv);
    let blur = apply_blur(&DynamicImage::ImageLuma8(inv), 10.0);
    let mut out = ImageBuffer::from_pixel(gray.width(), gray.height(), Luma([255]));
    for y in 0..gray.height() {
        for x in 0..gray.width() {
            let g = gray.get_pixel(x, y).0[0] as u32;
            let b = match blur {
                DynamicImage::ImageLuma8(ref bi) => bi.get_pixel(x, y).0[0] as u32,
                _ => 255,
            };
            let v = (g * 255) / (255 - b + 1);
            out.put_pixel(x, y, Luma([v.clamp(0, 255) as u8]));
        }
    }
    DynamicImage::ImageLuma8(out)
}

fn apply_hdr(img: &DynamicImage) -> DynamicImage {
    apply_contrast(img, 1.5)
}

fn apply_bokeh(img: &DynamicImage, radius: u32) -> DynamicImage {
    apply_blur(img, radius as f32 * 2.0)
}

fn apply_thermal(img: &DynamicImage) -> DynamicImage {
    let gray = image::imageops::grayscale(img);
    let mut out = ImageBuffer::from_pixel(gray.width(), gray.height(), Rgba([0, 0, 0, 255]));
    for y in 0..gray.height() {
        for x in 0..gray.width() {
            let v = gray.get_pixel(x, y).0[0] as f32 / 255.0;
            let (r, g, b) = if v < 0.25 {
                (0, 0, (v * 4.0 * 255.0) as u8)
            } else if v < 0.5 {
                (0, ((v - 0.25) * 4.0 * 255.0) as u8, 255)
            } else if v < 0.75 {
                (((v - 0.5) * 4.0 * 255.0) as u8, 255, (255.0 - (v - 0.5) * 4.0 * 255.0) as u8)
            } else {
                (255, (255.0 - (v - 0.75) * 4.0 * 255.0) as u8, 0)
            };
            out.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    DynamicImage::ImageRgba8(out)
}

fn apply_xray(img: &DynamicImage) -> DynamicImage {
    let mut out = img.clone();
    image::imageops::invert(&mut out);
    apply_tint(&out, Some("#00ffff"))
}

fn apply_infrared(img: &DynamicImage) -> DynamicImage {
    let mut out = img.to_rgba8();
    for y in 0..out.height() {
        for x in 0..out.width() {
            let p = out.get_pixel(x, y);
            out.get_pixel_mut(x, y).0 = [p.0[2], p.0[1], p.0[0], p.0[3]];
        }
    }
    DynamicImage::ImageRgba8(out)
}

fn apply_auto_enhance(img: &DynamicImage) -> DynamicImage {
    apply_normalize(img)
}

fn apply_remove_background(img: &DynamicImage) -> DynamicImage {
    apply_circle(img)
}

fn apply_vhs_effect(img: &DynamicImage) -> DynamicImage {
    let out = apply_chromatic(img, 0.5);
    apply_scanlines(&out, 0.3)
}

fn apply_neon_glow(img: &DynamicImage, amount: f32) -> DynamicImage {
    let _blur = apply_blur(img, amount * 4.0);
    apply_contrast(img, 1.5)
}

fn apply_polaroid_frame(img: &DynamicImage) -> DynamicImage {
    let w = img.width();
    let h = img.height();
    let border = (w.min(h) / 10).max(4);
    let mut canvas =
        ImageBuffer::from_pixel(w + border * 2, h + border * 3, Rgba([255, 255, 255, 255]));
    for y in 0..h {
        for x in 0..w {
            let p = img.get_pixel(x, y);
            canvas.put_pixel(x + border, y + border, Rgba([p.0[0], p.0[1], p.0[2], p.0[3]]));
        }
    }
    DynamicImage::ImageRgba8(canvas)
}

fn apply_watermark(img: &DynamicImage, _color: Option<&str>, opacity: f32) -> DynamicImage {
    let mut out = img.to_rgba8();
    let a = (opacity.clamp(0.0, 1.0) * 255.0) as u8;
    let text = b"WATERMARK";
    let tw = text.len() as u32 * 6;
    let th = 8;
    let x = out.width().saturating_sub(tw).saturating_sub(10);
    let y = out.height().saturating_sub(th).saturating_sub(10);
    for (i, ch) in text.iter().enumerate() {
        if *ch == 0 {
            continue;
        }
        let px = x + i as u32 * 6;
        if px < out.width() && y < out.height() {
            let p = out.get_pixel(px, y);
            out.put_pixel(
                px,
                y,
                Rgba([
                    (p.0[0] as u32 * (255 - a) as u32 / 255) as u8,
                    (p.0[1] as u32 * (255 - a) as u32 / 255) as u8,
                    (p.0[2] as u32 * (255 - a) as u32 / 255) as u8,
                    p.0[3],
                ]),
            );
        }
    }
    DynamicImage::ImageRgba8(out)
}

// ---------- shared helpers ----------

fn apply_edge_effect(img: &DynamicImage) -> DynamicImage {
    let gray = img.to_luma8();
    DynamicImage::ImageLuma16(gradients::sobel_gradients(&gray))
}

fn apply_posterize_colors(img: &DynamicImage, levels: u8) -> DynamicImage {
    let mut out = img.to_rgba8();
    let step = 256 / levels.max(1) as u32;
    for y in 0..out.height() {
        for x in 0..out.width() {
            let p = out.get_pixel_mut(x, y);
            for c in 0..3 {
                p.0[c] = (p.0[c] as u32 / step * step) as u8;
            }
        }
    }
    DynamicImage::ImageRgba8(out)
}

fn parse_hex_color(s: &str) -> Option<(u8, u8, u8)> {
    let s = s.trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some((r, g, b))
}
