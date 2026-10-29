pub mod editors;
pub mod effects;

use actix_web::web;

pub fn register(cfg: &mut web::ServiceConfig) {
    editors::register(cfg);
    effects::register(cfg);
}
