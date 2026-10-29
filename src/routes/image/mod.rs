pub mod editors;
pub mod effects;
pub mod meta;
pub mod convert;
pub mod collage;
pub mod watermark;

use actix_web::web;

pub fn register(cfg: &mut web::ServiceConfig) {
    editors::register(cfg);
    effects::register(cfg);
    meta::register(cfg);
    convert::register(cfg);
    collage::register(cfg);
    watermark::register(cfg);
}
