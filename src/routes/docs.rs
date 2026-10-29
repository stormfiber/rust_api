use actix_web::{web, HttpResponse, Responder};

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.route("/openapi.json", web::get().to(get_openapi));
    cfg.route("/docs", web::get().to(get_docs));
}

async fn get_openapi() -> impl Responder {
    let spec = serde_json::json!({
        "openapi": "3.0.3",
        "info": {
            "title": "Rust API - Image Tools",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "REST API for image processing with editors, effects, canvas, cards, QR, text, and URL endpoints."
        },
        "servers": [
            { "url": "http://127.0.0.1:3000" },
            { "url": "https://medium-enters-hall-rod.trycloudflare.com" }
        ],
        "paths": {
            "/health": {
                "get": {
                    "summary": "Health check",
                    "responses": { "200": { "description": "OK" } }
                }
            },
            "/image/meta": {
                "get": {
                    "summary": "Get image metadata",
                    "parameters": [
                        { "name": "image", "in": "query", "schema": { "type": "string" }, "description": "Base64-encoded image" },
                        { "name": "url", "in": "query", "schema": { "type": "string" }, "description": "Image URL" }
                    ],
                    "responses": { "200": { "description": "Image metadata" } }
                },
                "post": {
                    "summary": "Get image metadata (POST)",
                    "requestBody": { "content": { "application/json": { "schema": { "type": "object" } } } },
                    "responses": { "200": { "description": "Image metadata" } }
                }
            },
            "/image/convert": {
                "post": {
                    "summary": "Convert image format",
                    "parameters": [
                        { "name": "format", "in": "query", "schema": { "type": "string", "enum": ["png", "jpg", "webp"] }, "description": "Target format" },
                        { "name": "image", "in": "query", "schema": { "type": "string" }, "description": "Base64-encoded image" }
                    ],
                    "responses": { "200": { "description": "Converted image" } }
                }
            },
            "/image/collage": {
                "post": {
                    "summary": "Create image collage",
                    "requestBody": {
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "images": { "type": "array", "items": { "type": "string" } },
                                        "cols": { "type": "integer", "default": 2 },
                                        "background": { "type": "string", "default": "#ffffff" }
                                    }
                                }
                            }
                        }
                    },
                    "responses": { "200": { "description": "Collage image" } }
                }
            },
            "/image/watermark": {
                "post": {
                    "summary": "Add text watermark",
                    "parameters": [
                        { "name": "image", "in": "query", "schema": { "type": "string" } },
                        { "name": "text", "in": "query", "schema": { "type": "string", "default": "WATERMARK" } },
                        { "name": "position", "in": "query", "schema": { "type": "string", "default": "bottom-right" } },
                        { "name": "color", "in": "query", "schema": { "type": "string", "default": "#ffffff" } },
                        { "name": "opacity", "in": "query", "schema": { "type": "number", "default": 0.5 } },
                        { "name": "size", "in": "query", "schema": { "type": "integer", "default": 24 } }
                    ],
                    "responses": { "200": { "description": "Watermarked image" } }
                }
            },
            "/batch": {
                "post": {
                    "summary": "Batch image operations",
                    "requestBody": {
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "image": { "type": "string" },
                                        "ops": {
                                            "type": "array",
                                            "items": {
                                                "type": "object",
                                                "properties": {
                                                    "op": { "type": "string" },
                                                    "params": { "type": "object" }
                                                }
                                            }
                                        },
                                        "base64": { "type": "boolean", "default": true }
                                    },
                                    "required": ["image", "ops"]
                                }
                            }
                        }
                    },
                    "responses": { "200": { "description": "Processed image" } }
                }
            },
            "/auth/login": {
                "post": {
                    "summary": "Login and get JWT token",
                    "requestBody": {
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "username": { "type": "string" },
                                        "password": { "type": "string" }
                                    },
                                    "required": ["username", "password"]
                                }
                            }
                        }
                    },
                    "responses": { "200": { "description": "JWT token" } }
                }
            },
            "/auth/verify": {
                "post": {
                    "summary": "Verify JWT token",
                    "requestBody": {
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "token": { "type": "string" }
                                    },
                                    "required": ["token"]
                                }
                            }
                        }
                    },
                    "responses": { "200": { "description": "Verification result" } }
                }
            },
            "/metrics": {
                "get": {
                    "summary": "Prometheus metrics",
                    "responses": { "200": { "description": "Metrics in Prometheus text format" } }
                }
            }
        }
    });
    HttpResponse::Ok().content_type("application/json").json(spec)
}

async fn get_docs() -> impl Responder {
    let html = r##"<!DOCTYPE html>
<html>
<head>
    <title>Rust API - Swagger UI</title>
    <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5.9.0/swagger-ui.css" />
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5.9.0/swagger-ui-bundle.js"></script>
    <script>
        window.onload = function() {
            window.ui = SwaggerUIBundle({
                url: "/openapi.json",
                dom_id: "#swagger-ui",
                presets: [SwaggerUIBundle.presets.apis],
                layout: "StandaloneLayout"
            });
        };
    </script>
</body>
</html>"##;
    HttpResponse::Ok().content_type("text/html").body(html)
}
