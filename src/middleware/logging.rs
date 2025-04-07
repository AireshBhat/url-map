use std::future::{ready, Ready};
use std::pin::Pin;
use std::time::Instant;

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::Future;
use tracing::{info, error};

use crate::logging::generate_correlation_id;

pub struct RequestLogger;

impl<S, B> Transform<S, ServiceRequest> for RequestLogger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestLoggerMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestLoggerMiddleware { service }))
    }
}

pub struct RequestLoggerMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestLoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let correlation_id = generate_correlation_id();
        let start_time = Instant::now();
        let method = req.method().to_string();
        let uri = req.uri().to_string();
        let headers = format!("{:?}", req.headers());

        // Add correlation ID to request extensions
        req.extensions_mut().insert(correlation_id.clone());

        let fut = self.service.call(req);

        Box::pin(async move {
            let result = fut.await;
            let duration = start_time.elapsed();

            match &result {
                Ok(res) => {
                    info!(
                        correlation_id = %correlation_id,
                        method = %method,
                        uri = %uri,
                        status = %res.status().as_u16(),
                        duration_ms = %duration.as_millis(),
                        headers = %headers,
                        "Request completed"
                    );
                }
                Err(e) => {
                    error!(
                        correlation_id = %correlation_id,
                        method = %method,
                        uri = %uri,
                        error = %e,
                        duration_ms = %duration.as_millis(),
                        headers = %headers,
                        "Request failed"
                    );
                }
            }

            result
        })
    }
} 