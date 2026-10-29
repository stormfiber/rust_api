use actix_web::{http::StatusCode, HttpResponse};
use base64::{engine::general_purpose, Engine as _};
use image::DynamicImage;
use serde_json::json;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
pub enum ImageError {
    InvalidBase64(String),
    InvalidImageFormat(String),
    Io(String),
    #[allow(dead_code)]
    Other(String),
}

impl fmt::Display for ImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImageError::InvalidBase64(msg) => write!(f, "invalid base64 image: {msg}"),
            ImageError::InvalidImageFormat(msg) => write!(f, "invalid image format: {msg}"),
            ImageError::Io(msg) => write!(f, "image io error: {msg}"),
            ImageError::Other(msg) => write!(f, "image error: {msg}"),
        }
    }
}

impl std::error::Error for ImageError {}

impl From<std::io::Error> for ImageError {
    fn from(err: std::io::Error) -> Self {
        ImageError::Io(err.to_string())
    }
}

pub fn strip_data_uri_prefix(input: &str) -> &str {
    if let Some(idx) = input.find(";base64,") {
        &input[idx + ";base64,".len()..]
    } else {
        input
    }
}

pub fn decode_base64(input: &str) -> Result<Vec<u8>, ImageError> {
    let cleaned = strip_data_uri_prefix(input).trim();
    general_purpose::STANDARD
        .decode(cleaned)
        .map_err(|e| ImageError::InvalidBase64(e.to_string()))
}

pub fn load_from_bytes(bytes: &[u8]) -> Result<DynamicImage, ImageError> {
    image::load_from_memory(bytes).map_err(|e| ImageError::InvalidImageFormat(e.to_string()))
}

pub fn decode_base64_image(input: &str) -> Result<DynamicImage, ImageError> {
    let bytes = decode_base64(input)?;
    load_from_bytes(&bytes)
}

pub fn encode_png_bytes(image: &DynamicImage) -> Result<Vec<u8>, ImageError> {
    let mut buf = Vec::new();
    image
        .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .map_err(|e| ImageError::Io(e.to_string()))?;
    Ok(buf)
}

pub fn encode_base64_png(image: &DynamicImage) -> Result<String, ImageError> {
    let bytes = encode_png_bytes(image)?;
    Ok(general_purpose::STANDARD.encode(&bytes))
}

pub fn parse_u32(params: &std::collections::HashMap<String, String>, key: &str, default: u32) -> u32 {
    params
        .get(key)
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

pub fn parse_f32(params: &std::collections::HashMap<String, String>, key: &str, default: f32) -> f32 {
    params
        .get(key)
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

pub fn parse_str(
    params: &std::collections::HashMap<String, String>,
    key: &str,
    default: &str,
) -> String {
    params.get(key).cloned().unwrap_or_else(|| default.to_string())
}

pub fn error_response(err: &ImageError) -> HttpResponse {
    let status = match err {
        ImageError::InvalidBase64(_) | ImageError::InvalidImageFormat(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    HttpResponse::build(status).json(json!({ "error": err.to_string() }))
}

/// Merge a JSON body (object) into an existing params map. Non-JSON bodies are ignored.
pub fn merge_json_body(params: HashMap<String, String>, body: &[u8]) -> HashMap<String, String> {
    let mut p = params;
    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(body) {
        if let serde_json::Value::Object(map) = val {
            for (k, v) in map {
                let s = match v {
                    serde_json::Value::String(s) => s.clone(),
                    _ => v.to_string(),
                };
                p.insert(k, s);
            }
        }
    }
    p
}


