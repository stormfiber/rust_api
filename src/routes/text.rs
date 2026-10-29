use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use std::collections::HashMap;

use crate::utils::image::merge_json_body;

#[derive(Deserialize)]
pub struct TextRequest {
    pub message: String,
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.route("/text/echo", web::get().to(get_echo));
    cfg.route("/text/echo", web::post().to(post_echo));
    cfg.route("/text/reverse", web::get().to(get_reverse));
    cfg.route("/text/reverse", web::post().to(post_reverse));
}

fn message_from_params(params: &HashMap<String, String>) -> String {
    params.get("message").cloned().unwrap_or_else(|| "Hello Text".to_string())
}

// ----- GET -----

async fn get_echo(web::Query(params): web::Query<HashMap<String, String>>) -> impl Responder {
    let message = message_from_params(&params);
    HttpResponse::Ok().json(serde_json::json!({ "echo": message }))
}

async fn get_reverse(
    web::Query(params): web::Query<HashMap<String, String>>,
) -> impl Responder {
    let message = message_from_params(&params);
    let reversed: String = message.chars().rev().collect();
    HttpResponse::Ok().json(serde_json::json!({ "reversed": reversed }))
}

// ----- POST -----

async fn post_echo(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    let message = message_from_params(&params);
    HttpResponse::Ok().json(serde_json::json!({ "echo": message }))
}

async fn post_reverse(
    web::Query(params): web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> impl Responder {
    let params = merge_json_body(params, &body);
    let message = message_from_params(&params);
    let reversed: String = message.chars().rev().collect();
    HttpResponse::Ok().json(serde_json::json!({ "reversed": reversed }))
}
