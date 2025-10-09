mod qr;
mod text;
mod json;
mod url;

use actix_web::web;

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(qr::register_routes)
            .configure(text::register_routes)
            .configure(json::register_routes)
            .configure(url::register_routes)
    );
}
