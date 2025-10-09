use actix_web::{App, HttpServer, middleware};
mod routes;
mod middleware as my_middleware;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(my_middleware::cache::CacheMiddleware)
            .configure(routes::register_routes)
    })
    .bind(("0.0.0.0", port.parse().unwrap()))?
    .run()
    .await
}
