use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_files::Files;

mod config;
mod routes;
mod middleware;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let config = config::Config::from_env();
    routes::health::init_start_time();

    // Start keep-alive ping in background
    tokio::spawn(utils::keep_alive());

    let config_for_bind = config.clone();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            // Custom middleware (before Logger so body type matches BoxBody)
            .wrap(middleware::metrics_timer::MetricsTimerMiddleware::new())
            .wrap(middleware::rate_limit::RateLimitMiddleware::new(config.clone()))
            .wrap(middleware::api_key::ApiKeyMiddleware::new(config.api_key.clone()))
            // Standard middleware
            .wrap(Logger::default())
            // Custom error handlers
            .wrap(middleware::error_handlers())
            // Static files
            .service(Files::new("/public", "./public").show_files_listing())
            // Register modular routes
            .configure(routes::register_routes)
    })
    .bind(config_for_bind.addr())?
    .run()
    .await
}
