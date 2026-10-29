use actix_web::{web, HttpResponse, Responder};
use serde::Serialize;
use std::collections::HashMap;

use crate::utils::image::{self, merge_json_body};
use ::image::{DynamicImage, GenericImageView};

#[derive(Serialize)]
pub struct MetaResponse {
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub color_type: String,
    pub size_bytes: usize,
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.route("/image/meta", web::get().to(get_meta));
    cfg.route("/image/meta", web::post().to(post_meta));
}

async fn get_meta(web::Query(params): web::Query<HashMap<String, String>>) -> impl Responder {
    core_meta(&params, None)
}

async fn post_meta(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_meta(&params, Some(&body))
}

fn core_meta(params: &HashMap<String, String>, body: Option<&[u8]>) -> impl Responder {
    let img = if let Some(b64) = params.get("image").filter(|s| !s.is_empty()) {
        match image::decode_base64_image(b64) {
            Ok(i) => i,
            Err(e) => return image::error_response(&e),
        }
    } else if let Some(b) = body.filter(|b| !b.is_empty()) {
        match image::load_from_bytes(b) {
            Ok(i) => i,
            Err(e) => return image::error_response(&e),
        }
    } else {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "image or url is required" }));
    };

    let (width, height) = img.dimensions();
    let color_type = match &img {
        DynamicImage::ImageLuma8(_) => "Luma8",
        DynamicImage::ImageLuma16(_) => "Luma16",
        DynamicImage::ImageRgb8(_) => "Rgb8",
        DynamicImage::ImageRgba8(_) => "Rgba8",
        DynamicImage::ImageRgb16(_) => "Rgb16",
        DynamicImage::ImageRgba16(_) => "Rgba16",
        DynamicImage::ImageRgb32F(_) => "Rgb32F",
        DynamicImage::ImageRgba32F(_) => "Rgba32F",
        _ => "Unknown",
    };

    let size_bytes = match image::encode_png_bytes(&img) {
        Ok(b) => b.len(),
        Err(_) => 0,
    };

    HttpResponse::Ok().json(MetaResponse {
        width,
        height,
        format: "png".to_string(),
        color_type: color_type.to_string(),
        size_bytes,
    })
}
