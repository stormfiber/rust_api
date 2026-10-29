pub mod qr;
pub mod text;
pub mod url;
pub mod image;
pub mod canvas;
pub mod cards;
pub mod health;
pub mod auth;
pub mod batch;
pub mod metrics;
pub mod docs;

use actix_web::web;

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    qr::register(cfg);
    text::register(cfg);
    url::register(cfg);
    image::register(cfg);
    canvas::register(cfg);
    cards::register(cfg);
    health::register(cfg);
    auth::register(cfg);
    batch::register(cfg);
    metrics::register(cfg);
    docs::register(cfg);
}

#[cfg(test)]
mod tests {
    use actix_web::{http, test, App};
    use base64::{engine::general_purpose, Engine as _};
    use serde_json::json;

    const PNG_B64: &str =
        "iVBORw0KGgoAAAANSUhEUgAAAAQAAAAECAYAAACp8Z5+AAAAR0lEQVR4Ae3AA6AkWZbG8f937o3IzKdyS2Oubdu2bdu2bdu2bWmMnpZKr54yMyLu+Xa3anqmhztr1a8azLMRPCeC50TwnPhHhJ0CCRuWqnkAAAAASUVORK5CYII=";

    async fn call(req: test::TestRequest) -> actix_web::dev::ServiceResponse {
        let mut app =
            test::init_service(App::new().configure(super::register_routes)).await;
        test::call_service(&mut app, req.to_request()).await
    }

    fn png_bytes() -> Vec<u8> {
        general_purpose::STANDARD.decode(PNG_B64).unwrap()
    }

    async fn post_json(uri: &str, value: serde_json::Value) -> actix_web::dev::ServiceResponse {
        let req = test::TestRequest::post().uri(uri).set_json(&value);
        call(req).await
    }

    #[actix_web::test]
    async fn grayscale_base64() {
        let resp = post_json(
            "/image/editors/grayscale_base64",
            json!({"image": PNG_B64}),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        let body = test::read_body(resp).await;
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(value.get("image_base64").is_some());
    }

    #[actix_web::test]
    async fn grayscale_raw() {
        let req = test::TestRequest::post()
            .uri("/image/editors/grayscale")
            .set_payload(png_bytes());
        let resp = call(req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        assert_eq!(
            resp.headers().get(http::header::CONTENT_TYPE).unwrap(),
            http::header::HeaderValue::from_static("image/png")
        );
    }

    #[actix_web::test]
    async fn resize_base64() {
        let resp = post_json(
            "/image/editors/resize_base64",
            json!({"image": PNG_B64, "width": 8, "height": 8}),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        let body = test::read_body(resp).await;
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(value.get("image_base64").is_some());
    }

    #[actix_web::test]
    async fn resize_missing_dimensions() {
        let resp = post_json(
            "/image/editors/resize_base64",
            json!({"image": PNG_B64}),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn invert_base64() {
        let resp = post_json(
            "/image/effects/invert_base64",
            json!({"image": PNG_B64}),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        let body = test::read_body(resp).await;
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(value.get("image_base64").is_some());
    }

    #[actix_web::test]
    async fn sepia_base64() {
        let resp = post_json(
            "/image/effects/sepia_base64",
            json!({"image": PNG_B64}),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        let body = test::read_body(resp).await;
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(value.get("image_base64").is_some());
    }

    #[actix_web::test]
    async fn edge_base64() {
        let resp = post_json(
            "/image/effects/edge_base64",
            json!({"image": PNG_B64}),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        let body = test::read_body(resp).await;
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(value.get("image_base64").is_some());
    }

    #[actix_web::test]
    async fn blur_base64() {
        let resp = post_json(
            "/image/editors/blur_base64",
            json!({"image": PNG_B64, "sigma": 2.0}),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        let body = test::read_body(resp).await;
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(value.get("image_base64").is_some());
    }

    #[actix_web::test]
    async fn flip_base64() {
        let resp = post_json(
            "/image/editors/flip_base64",
            json!({"image": PNG_B64, "direction": "vertical"}),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        let body = test::read_body(resp).await;
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(value.get("image_base64").is_some());
    }

    #[actix_web::test]
    async fn rotate_base64() {
        let resp = post_json(
            "/image/editors/rotate_base64",
            json!({"image": PNG_B64, "deg": 90}),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        let body = test::read_body(resp).await;
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(value.get("image_base64").is_some());
    }

    #[actix_web::test]
    async fn canvas_blank_base64() {
        let resp = post_json(
            "/canvas/blank_base64",
            json!({"width": 20, "height": 10, "color": "blue"}),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        let body = test::read_body(resp).await;
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(value.get("image_base64").is_some());
    }

    #[actix_web::test]
    async fn canvas_blank_raw() {
        let req = test::TestRequest::post()
            .uri("/canvas/blank?width=8&height=4&color=red");
        let resp = call(req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn canvas_draw_text_base64() {
        let resp = post_json(
            "/canvas/draw_text_base64",
            json!({"text": "Hello", "width": 80, "height": 30, "size": 16}),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        let body = test::read_body(resp).await;
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(value.get("image_base64").is_some());
    }

    #[actix_web::test]
    async fn cards_quote_base64() {
        let resp = post_json(
            "/cards/quote_base64",
            json!({"text": "Hello world", "text_color": "#ffd700"}),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        let body = test::read_body(resp).await;
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(value.get("image_base64").is_some());
    }

    #[actix_web::test]
    async fn cards_quote_default_text() {
        let resp = post_json(
            "/cards/quote_base64",
            json!({}),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        let body = test::read_body(resp).await;
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(value.get("image_base64").is_some());
    }

    #[actix_web::test]
    async fn cards_welcome_base64() {
        let resp = post_json(
            "/cards/welcome_base64",
            json!({"text": "Welcome aboard!", "background": "#1a1a2e"}),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        let body = test::read_body(resp).await;
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(value.get("image_base64").is_some());
    }

    // ----- comprehensive battle tests -----

    async fn assert_base64_ok(uri: &str, body: serde_json::Value) {
        let resp = post_json(uri, body).await;
        assert_eq!(
            resp.status(),
            http::StatusCode::OK,
            "{} returned {:?}",
            uri,
            resp.status()
        );
        let body = test::read_body(resp).await;
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(
            value.get("image_base64").is_some(),
            "{} missing image_base64",
            uri
        );
    }

    async fn assert_raw_png_ok(uri: &str, query: &str, body: Vec<u8>) {
        let full_uri = if query.is_empty() {
            uri.to_string()
        } else {
            format!("{}?{}", uri, query)
        };
        let req = test::TestRequest::post().uri(&full_uri).set_payload(body);
        let resp = call(req).await;
        assert_eq!(
            resp.status(),
            http::StatusCode::OK,
            "{}?{} returned {:?}",
            uri,
            query,
            resp.status()
        );
        assert_eq!(
            resp.headers().get(http::header::CONTENT_TYPE).unwrap(),
            http::header::HeaderValue::from_static("image/png")
        );
    }

    // ---- all 18 card types via base64 JSON (defaults) ----
    #[actix_web::test]
    async fn all_card_endpoints_base64() {
        let paths = [
            "/cards/welcome_base64", "/cards/rank_base64", "/cards/leave_base64",
            "/cards/profile_base64", "/cards/birthday_base64", "/cards/achievement_base64",
            "/cards/quote_base64", "/cards/spotify_base64", "/cards/github_stats_base64",
            "/cards/warning_base64", "/cards/ban_base64", "/cards/announcement_base64",
            "/cards/poll_vote_base64", "/cards/ship_base64", "/cards/magic8ball_base64",
            "/cards/weather_base64", "/cards/certificate_base64",
            "/cards/support_ticket_base64",
        ];
        for uri in paths {
            assert_base64_ok(uri, serde_json::json!({})).await;
        }
    }

    // ---- card raw endpoints via query params ----
    #[actix_web::test]
    async fn cards_welcome_raw() {
        assert_raw_png_ok(
            "/cards/welcome",
            "text=Hi&background=#1a1a2e&text_color=#00d2ff",
            Vec::new(),
        )
        .await;
    }

    #[actix_web::test]
    async fn cards_quote_missing_image_is_ok() {
        assert_base64_ok("/cards/quote_base64", serde_json::json!({})).await;
    }

    // ---- image editors: base64 JSON ----
    #[actix_web::test]
    async fn editors_resize_base64() {
        assert_base64_ok(
            "/image/editors/resize_base64",
            json!({"image": PNG_B64, "width": 16, "height": 16, "mode": "fit"}),
        )
        .await;
    }

    #[actix_web::test]
    async fn editors_resize_exact_base64() {
        assert_base64_ok(
            "/image/editors/resize_base64",
            json!({"image": PNG_B64, "width": 32, "height": 32, "mode": "exact"}),
        )
        .await;
    }

    #[actix_web::test]
    async fn editors_resize_fill_base64() {
        assert_base64_ok(
            "/image/editors/resize_base64",
            json!({"image": PNG_B64, "width": 32, "height": 32, "mode": "fill"}),
        )
        .await;
    }

    #[actix_web::test]
    async fn editors_resize_missing_dims_400() {
        let resp = post_json("/image/editors/resize_base64", json!({"image": PNG_B64})).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn editors_rotate_base64() {
        assert_base64_ok(
            "/image/editors/rotate_base64",
            json!({"image": PNG_B64, "deg": 90}),
        )
        .await;
    }

    #[actix_web::test]
    async fn editors_flip_base64() {
        assert_base64_ok(
            "/image/editors/flip_base64",
            json!({"image": PNG_B64, "direction": "vertical"}),
        )
        .await;
    }

    #[actix_web::test]
    async fn editors_grayscale_base64() {
        assert_base64_ok("/image/editors/grayscale_base64", json!({"image": PNG_B64})).await;
    }

    #[actix_web::test]
    async fn editors_blur_base64() {
        assert_base64_ok(
            "/image/editors/blur_base64",
            json!({"image": PNG_B64, "sigma": 2.5}),
        )
        .await;
    }

    // ---- image editors: raw bytes ----
    #[actix_web::test]
    async fn editors_grayscale_raw() {
        assert_raw_png_ok("/image/editors/grayscale", "", png_bytes()).await;
    }

    #[actix_web::test]
    async fn editors_blur_raw() {
        assert_raw_png_ok("/image/editors/blur", "sigma=2.0", png_bytes()).await;
    }

    #[actix_web::test]
    async fn editors_resize_raw() {
        assert_raw_png_ok("/image/editors/resize", "width=8&height=8", png_bytes()).await;
    }

    #[actix_web::test]
    async fn editors_resize_raw_missing_dims_400() {
        let req = test::TestRequest::post()
            .uri("/image/editors/resize")
            .set_payload(png_bytes());
        let resp = call(req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }

    // ---- image effects: comprehensive /image/effects_base64 ----
    #[actix_web::test]
    async fn all_effects_endpoints_base64() {
        let effects = [
            "blur", "brightness", "contrast", "sharpen", "grayscale", "invert",
            "normalize", "sepia", "flip", "resize", "crop", "scale", "contain",
            "cover", "opacity", "fade", "saturation", "hue", "correction",
            "threshold", "tint", "adjustment", "median", "pixelate", "convolute",
            "gaussian", "posturize", "dither", "circle", "fisheye", "drop_shadow",
            "displace", "mirror", "glitch", "vignette", "vintage", "emboss",
            "oil_painting", "duo_tone", "chromatic", "halftone", "ascii_art",
            "scanlines", "motion_blur", "kaleidoscope", "pixel_sort",
            "wave_distortion", "swirl", "bulge", "noise", "denoise", "cartoon",
            "pencil_sketch", "hdr", "bokeh", "thermal", "xray", "infrared",
            "auto_enhance", "remove_background", "vhs_effect", "neon_glow",
            "polaroid_frame", "watermark",
        ];
        for eff in effects {
            let mut body =
                serde_json::json!({"image": PNG_B64, "effect": eff});
            if eff == "threshold" {
                body["threshold"] = json!(128);
            }
            if eff == "tint" || eff == "duo_tone" || eff == "watermark" {
                body["color"] = json!("#ff0000");
            }
            if eff == "blur" || eff == "gaussian" {
                body["sigma"] = json!(2.0);
            }
            if eff == "hue" {
                body["angle"] = json!(45.0);
            }
            if eff == "posturize" {
                body["amount"] = json!(4);
            }
            if eff == "median" || eff == "pixelate" || eff == "circle"
                || eff == "oil_painting" || eff == "halftone" || eff == "bokeh"
            {
                body["radius"] = json!(4);
            }
            if eff == "drop_shadow" {
                body["x"] = json!(10);
                body["y"] = json!(10);
                body["radius"] = json!(8);
                body["opacity"] = json!(0.5);
            }
            assert_base64_ok("/image/effects_base64", body).await;
        }
    }

    // ---- image effects: raw bytes endpoint ----
    #[actix_web::test]
    async fn effects_raw_sepia() {
        let req = test::TestRequest::post()
            .uri("/image/effects?effect=sepia")
            .set_payload(png_bytes());
        let resp = call(req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        assert_eq!(
            resp.headers().get(http::header::CONTENT_TYPE).unwrap(),
            http::header::HeaderValue::from_static("image/png")
        );
    }

    #[actix_web::test]
    async fn effects_raw_invert() {
        let req = test::TestRequest::post()
            .uri("/image/effects?effect=invert")
            .set_payload(png_bytes());
        let resp = call(req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn effects_raw_brightness() {
        let req = test::TestRequest::post()
            .uri("/image/effects?effect=brightness&amount=1.5")
            .set_payload(png_bytes());
        let resp = call(req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn effects_missing_image_400() {
        let resp = post_json(
            "/image/effects_base64",
            json!({"effect": "blur"}),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }

    // ---- legacy effect endpoints ----
    #[actix_web::test]
    async fn legacy_sepia_base64() {
        assert_base64_ok(
            "/image/effects/sepia_base64",
            json!({"image": PNG_B64}),
        )
        .await;
    }

    #[actix_web::test]
    async fn legacy_invert_base64() {
        assert_base64_ok(
            "/image/effects/invert_base64",
            json!({"image": PNG_B64}),
        )
        .await;
    }

    #[actix_web::test]
    async fn legacy_edge_base64() {
        assert_base64_ok(
            "/image/effects/edge_base64",
            json!({"image": PNG_B64}),
        )
        .await;
    }

    // ---- canvas ----
    #[actix_web::test]
    async fn canvas_blank_missing_dims_400() {
        let resp = post_json("/canvas/blank_base64", serde_json::json!({})).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }

    // ---- effects_get: missing url -> 400 ----
    #[actix_web::test]
    async fn effects_get_missing_url_400() {
        let req = test::TestRequest::get().uri("/image/effects_get");
        let resp = call(req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }
}

