use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::Config;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_in: i64,
}

#[derive(Serialize)]
pub struct VerifyResponse {
    pub valid: bool,
    pub username: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.route("/auth/login", web::post().to(post_login));
    cfg.route("/auth/verify", web::post().to(post_verify));
}

async fn post_login(
    cfg: web::Data<Config>,
    body: web::Json<LoginRequest>,
) -> impl Responder {
    // Simple demo auth: accept any non-empty credentials
    if body.username.is_empty() || body.password.is_empty() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "username and password are required" }));
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
    let exp = now + 3600; // 1 hour

    let claims = Claims {
        sub: body.username.clone(),
        exp,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(cfg.jwt_secret.as_bytes()),
    )
    .map_err(|e| {
        HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": format!("token creation failed: {e}") }))
    });

    match token {
        Ok(t) => HttpResponse::Ok().json(LoginResponse {
            token: t,
            expires_in: exp - now,
        }),
        Err(e) => e,
    }
}

async fn post_verify(
    cfg: web::Data<Config>,
    body: web::Json<serde_json::Value>,
) -> impl Responder {
    let token = body.get("token")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if token.is_empty() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "token is required" }));
    }

    let result = decode::<Claims>(
        token,
        &DecodingKey::from_secret(cfg.jwt_secret.as_bytes()),
        &Validation::default(),
    );

    match result {
        Ok(data) => HttpResponse::Ok().json(VerifyResponse {
            valid: true,
            username: Some(data.claims.sub),
        }),
        Err(_) => HttpResponse::Ok().json(VerifyResponse {
            valid: false,
            username: None,
        }),
    }
}

pub fn extract_token_from_req(req: &actix_web::HttpRequest) -> Option<String> {
    req.headers()
        .get(actix_web::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

pub fn validate_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|d| d.claims)
}
