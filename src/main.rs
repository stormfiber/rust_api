use actix_web::{App, HttpServer, middleware::Logger};
use actix_files::Files;

mod routes;
mod middleware;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // Start keep-alive ping in background
    tokio::spawn(utils::keep_alive());

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            // Custom error handlers
            .wrap(middleware::error_handlers())
            // Static files
            .service(Files::new("/public", "./public").show_files_listing())
            // Register modular routes
            .configure(routes::register_routes)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
