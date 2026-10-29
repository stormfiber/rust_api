use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    Error,
};
use std::future::{ready, Future};
use std::pin::Pin;
use std::task::{Context, Poll};
use actix_web::dev::{Service, Transform};
use std::time::Instant;
use crate::routes::metrics::record_request;

pub struct MetricsTimerMiddleware;

impl MetricsTimerMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Transform<S, ServiceRequest> for MetricsTimerMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = MetricsTimerMiddlewareService<S>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MetricsTimerMiddlewareService { service }))
    }
}

pub struct MetricsTimerMiddlewareService<S> {
    service: S,
}

impl<S> Service<ServiceRequest> for MetricsTimerMiddlewareService<S>
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
        let start = Instant::now();
        let method = req.method().as_str().to_string();
        let path = req.path().to_string();
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            let status = res.status().as_u16();
            let duration_secs = start.elapsed().as_secs_f64();
            record_request(&method, &path, status, duration_secs);
            Ok(res)
        })
    }
}
