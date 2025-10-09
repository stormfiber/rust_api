use actix_web::{dev::ServiceResponse, middleware::ErrorHandlerResponse, http::StatusCode};

pub fn error_handlers<B>() -> actix_web::middleware::ErrorHandlers<B> {
    actix_web::middleware::ErrorHandlers::new()
        .handler(StatusCode::NOT_FOUND, not_found)
        .handler(StatusCode::INTERNAL_SERVER_ERROR, internal_error)
}

fn not_found<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>, actix_web::Error> {
    let response = actix_web::HttpResponse::NotFound()
        .json(serde_json::json!({"error": "Not Found"}));
    Ok(ErrorHandlerResponse::Response(
        res.into_response(response.map_into_right_body())
    ))
}

fn internal_error<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>, actix_web::Error> {
    let response = actix_web::HttpResponse::InternalServerError()
        .json(serde_json::json!({"error": "Internal Server Error"}));
    Ok(ErrorHandlerResponse::Response(
        res.into_response(response.map_into_right_body())
    ))
}
