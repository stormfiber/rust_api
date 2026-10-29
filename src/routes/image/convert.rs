use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Cursor;

use crate::utils::image::{self, merge_json_body};
use ::image::ImageFormat;

#[derive(Deserialize)]
pub struct ConvertQuery {
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "png".to_string()
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.route("/image/convert", web::get().to(get_convert));
    cfg.route("/image/convert", web::post().to(post_convert));
}

async fn get_convert(
    web::Query(query): web::Query<ConvertQuery>,
    web::Query(params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    core_convert(&query.format, &params, None)
}

async fn post_convert(
    web::Query(query): web::Query<ConvertQuery>,
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    core_convert(&query.format, &params, Some(&body))
}

fn core_convert(format: &str, params: &HashMap<String, String>, body: Option<&[u8]>) -> impl Responder {
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

    let content_type = match format {
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        _ => "image/png",
    };

    let mut buf = Vec::new();
    let result = match format {
        "jpg" | "jpeg" => img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg),
        "webp" => img.write_to(&mut Cursor::new(&mut buf), ImageFormat::WebP),
        _ => img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png),
    };

    match result {
        Ok(_) => HttpResponse::Ok().content_type(content_type).body(buf),
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": format!("encode failed: {e}") })),
    }
}
