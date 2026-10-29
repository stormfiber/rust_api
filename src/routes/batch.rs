use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

use crate::utils::image::{self};
use ::image::{DynamicImage, ImageBuffer, Luma, Rgba, imageops};

#[derive(Deserialize)]
pub struct BatchOp {
    pub op: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Deserialize)]
pub struct BatchPayload {
    pub image: String,
    pub ops: Vec<BatchOp>,
    #[serde(default = "default_base64")]
    pub base64: bool,
}

fn default_base64() -> bool {
    true
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.route("/batch", web::post().to(post_batch));
}

async fn post_batch(
    payload: web::Json<BatchPayload>,
) -> impl Responder {
    core_batch_payload(payload.into_inner())
}

fn core_batch_payload(payload: BatchPayload) -> HttpResponse {

    if payload.ops.is_empty() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "ops array is required" }));
    }

    let mut img = match image::decode_base64_image(&payload.image) {
        Ok(i) => i,
        Err(e) => return image::error_response(&e),
    };

    for op in &payload.ops {
        img = apply_op(&img, op);
    }

    if payload.base64 {
        match image::encode_base64_png(&img) {
            Ok(b64) => HttpResponse::Ok().json(serde_json::json!({ "image_base64": b64 })),
            Err(e) => image::error_response(&e),
        }
    } else {
        match image::encode_png_bytes(&img) {
            Ok(bytes) => HttpResponse::Ok().content_type("image/png").body(bytes),
            Err(e) => image::error_response(&e),
        }
    }
}

fn apply_op(img: &DynamicImage, op: &BatchOp) -> DynamicImage {
    match op.op.as_str() {
        "grayscale" => DynamicImage::ImageLuma8(imageops::grayscale(img)),
        "invert" => {
            let mut out = img.clone();
            imageops::invert(&mut out);
            out
        }
        "flip" => {
            let rgba = img.to_rgba8();
            DynamicImage::ImageRgba8(imageops::flip_horizontal(&rgba))
        }
        "flip_vertical" => {
            let rgba = img.to_rgba8();
            DynamicImage::ImageRgba8(imageops::flip_vertical(&rgba))
        }
        "rotate90" => {
            let rgba = img.to_rgba8();
            DynamicImage::ImageRgba8(imageops::rotate90(&rgba))
        }
        "rotate180" => {
            let rgba = img.to_rgba8();
            DynamicImage::ImageRgba8(imageops::rotate180(&rgba))
        }
        "rotate270" => {
            let rgba = img.to_rgba8();
            DynamicImage::ImageRgba8(imageops::rotate270(&rgba))
        }
        "blur" => {
            let sigma: f32 = op.params.get("sigma").and_then(|v| v.as_f64()).unwrap_or(2.0) as f32;
            let rgba = img.to_rgba8();
            if sigma <= 0.0 {
                DynamicImage::ImageRgba8(rgba)
            } else {
                DynamicImage::ImageRgba8(imageproc::filter::gaussian_blur_f32(&rgba, sigma))
            }
        }
        "sepia" => apply_sepia(img),
        "resize" => {
            let w: u32 = op.params.get("width")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32)
                .unwrap_or(img.width());
            let h: u32 = op.params.get("height")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32)
                .unwrap_or(img.height());
            DynamicImage::ImageRgba8(imageops::resize(
                img,
                w.max(1),
                h.max(1),
                ::image::imageops::FilterType::Triangle,
            ))
        }
        "threshold" => {
            let threshold: u8 = op.params.get("threshold").and_then(|v| v.as_u64()).unwrap_or(128) as u8;
            let gray = imageops::grayscale(img);
            let mut out = ImageBuffer::from_pixel(img.width(), img.height(), Luma([0]));
            for y in 0..gray.height() {
                for x in 0..gray.width() {
                    let v = gray.get_pixel(x, y).0[0];
                    out.put_pixel(x, y, Luma([if v >= threshold { 255 } else { 0 }]));
                }
            }
            DynamicImage::ImageLuma8(out)
        }
        _ => img.clone(),
    }
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
            let nr = (0.393 * r + 0.769 * g + 0.189 * b).min(255.0) as u8;
            let ng = (0.349 * r + 0.686 * g + 0.168 * b).min(255.0) as u8;
            let nb = (0.272 * r + 0.534 * g + 0.131 * b).min(255.0) as u8;
            out.put_pixel(x, y, Rgba([nr, ng, nb, p.0[3]]));
        }
    }
    DynamicImage::ImageRgba8(out)
}
