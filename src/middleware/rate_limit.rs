use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    Error,
};
use std::future::{ready, Future};
use std::pin::Pin;
use std::task::{Context, Poll};
use actix_web::dev::{Service, Transform};
use serde_json::json;

use crate::config::Config;

pub struct RateLimitMiddleware {
    config: Config,
    redis_client: redis::Client,
}

impl RateLimitMiddleware {
    pub fn new(config: Config) -> Self {
        let redis_client = redis::Client::open(config.redis_url.clone())
            .unwrap_or_else(|_| redis::Client::open("redis://127.0.0.1:6379").unwrap());
        Self { config, redis_client }
    }
}

impl<S> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddlewareService<S>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddlewareService {
            service,
            config: self.config.clone(),
            redis_client: self.redis_client.clone(),
        }))
    }
}

pub struct RateLimitMiddlewareService<S> {
    service: S,
    config: Config,
    redis_client: redis::Client,
}

impl<S> Service<ServiceRequest> for RateLimitMiddlewareService<S>
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
        let config = self.config.clone();
        let redis_client = self.redis_client.clone();
        let peer_addr = req.peer_addr();
        let fut = self.service.call(req);

        Box::pin(async move {
            let client_ip = peer_addr
                .map(|addr| addr.ip().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let key = format!("ratelimit:{}", client_ip);
            let max = config.rate_limit_max;
            let window = config.rate_limit_window_secs;

            let allowed = match redis_client.get_async_connection().await {
                Ok(mut conn) => {
                    let count: i64 = match redis::cmd("INCR").arg(&key).query_async(&mut conn).await {
                        Ok(c) => c,
                        Err(_) => 1,
                    };

                    if count == 1 {
                        let _: Result<(), _> = redis::cmd("EXPIRE").arg(&key).arg(window).query_async(&mut conn).await;
                    }

                    count <= max as i64
                }
                Err(_) => true,
            };

            if !allowed {
                return Err(actix_web::error::ErrorTooManyRequests(
                    serde_json::to_string(&json!({
                        "error": "rate limit exceeded",
                        "limit": max,
                        "window_secs": window
                    })).unwrap(),
                ));
            }

            fut.await
        })
    }
}
