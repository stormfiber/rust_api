use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    http::header,
    Error, HttpResponse,
};
use std::future::{ready, Future};
use std::pin::Pin;
use std::task::{Context, Poll};
use actix_web::dev::{Service, Transform};
use serde_json::json;

pub struct ApiKeyMiddleware {
    api_key: Option<String>,
}

impl ApiKeyMiddleware {
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }
}

impl<S> Transform<S, ServiceRequest> for ApiKeyMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = ApiKeyMiddlewareService<S>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ApiKeyMiddlewareService {
            service,
            api_key: self.api_key.clone(),
        }))
    }
}

pub struct ApiKeyMiddlewareService<S> {
    service: S,
    api_key: Option<String>,
}

impl<S> Service<ServiceRequest> for ApiKeyMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<ServiceResponse, Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if let Some(expected_key) = &self.api_key {
            let provided = req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .or_else(|| req.headers()
                    .get(header::HeaderName::from_static("x-api-key"))
                    .and_then(|v| v.to_str().ok()));

            if provided != Some(expected_key.as_str()) {
                return Box::pin(async move {
                    Ok(req.into_response(
                        HttpResponse::Unauthorized()
                            .json(serde_json::json!({ "error": "invalid or missing API key" }))
                    ))
                });
            }
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            fut.await
        })
    }
}
