pub mod qr;
pub mod text;
pub mod url;

use actix_web::web;

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    qr::register(cfg);
    text::register(cfg);
    url::register(cfg);
}
