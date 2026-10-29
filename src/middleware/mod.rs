use actix_web::{
    dev::ServiceResponse,
    middleware::ErrorHandlerResponse,
    middleware::ErrorHandlers,
    http::StatusCode,
    HttpResponse,
    Error,
};
use serde_json::json;

pub mod api_key;
pub mod rate_limit;
pub mod metrics_timer;

pub fn error_handlers<B: 'static>() -> ErrorHandlers<B> {
    ErrorHandlers::new()
        .handler(StatusCode::NOT_FOUND, not_found)
        .handler(StatusCode::INTERNAL_SERVER_ERROR, internal_error)
}

fn not_found<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>, Error> {
    let response = HttpResponse::NotFound()
        .json(json!({"error": "Not Found"}));
    Ok(ErrorHandlerResponse::Response(
        res.into_response(response.map_into_right_body())
    ))
}

fn internal_error<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>, Error> {
    let response = HttpResponse::InternalServerError()
        .json(json!({"error": "Internal Server Error"}));
    Ok(ErrorHandlerResponse::Response(
        res.into_response(response.map_into_right_body())
    ))
}
