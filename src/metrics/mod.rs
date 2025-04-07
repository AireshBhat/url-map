#[cfg(feature = "metrics")]
use lazy_static::lazy_static;
#[cfg(feature = "metrics")]
use prometheus::{
    register_counter, register_gauge, register_histogram,
    Counter, Gauge, Histogram, TextEncoder,
};
use std::time::Duration;

pub use middleware::MetricsMiddleware;

mod middleware;

#[cfg(feature = "metrics")]
lazy_static! {
    // URL Shortening Metrics
    pub static ref TOTAL_SHORTEN_REQUESTS: Counter = register_counter!(
        "url_shortener_total_requests",
        "Total number of URL shortening requests"
    ).unwrap();
    
    pub static ref SUCCESSFUL_SHORTENINGS: Counter = register_counter!(
        "url_shortener_successful_shortenings",
        "Number of successful URL shortenings"
    ).unwrap();
    
    pub static ref FAILED_SHORTENINGS: Counter = register_counter!(
        "url_shortener_failed_shortenings",
        "Number of failed URL shortenings"
    ).unwrap();
    
    pub static ref SHORTENING_LATENCY: Histogram = register_histogram!(
        "url_shortener_shortening_latency_seconds",
        "Time taken to shorten URLs",
        vec![0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0]
    ).unwrap();
    
    pub static ref ACTIVE_SHORT_URLS: Gauge = register_gauge!(
        "url_shortener_active_short_urls",
        "Number of active short URLs"
    ).unwrap();

    // Redirect Metrics
    pub static ref TOTAL_REDIRECTS: Counter = register_counter!(
        "url_shortener_total_redirects",
        "Total number of redirect requests"
    ).unwrap();
    
    pub static ref SUCCESSFUL_REDIRECTS: Counter = register_counter!(
        "url_shortener_successful_redirects",
        "Number of successful redirects"
    ).unwrap();
    
    pub static ref FAILED_REDIRECTS: Counter = register_counter!(
        "url_shortener_failed_redirects",
        "Number of failed redirects"
    ).unwrap();
    
    pub static ref REDIRECT_LATENCY: Histogram = register_histogram!(
        "url_shortener_redirect_latency_seconds",
        "Time taken to process redirects",
        vec![0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0]
    ).unwrap();

    // HTTP Metrics
    pub static ref HTTP_REQUESTS_TOTAL: Counter = register_counter!(
        "url_shortener_http_requests_total",
        "Total number of HTTP requests"
    ).unwrap();
    
    pub static ref HTTP_REQUEST_DURATION: Histogram = register_histogram!(
        "url_shortener_http_request_duration_seconds",
        "HTTP request duration in seconds",
        vec![0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0]
    ).unwrap();
    
    pub static ref HTTP_RESPONSE_STATUS: Counter = register_counter!(
        "url_shortener_http_response_status_codes",
        "HTTP response status codes"
    ).unwrap();
}

#[cfg(feature = "metrics")]
pub fn init_metrics() -> anyhow::Result<()> {
    // Register process metrics only on Linux
    #[cfg(all(feature = "metrics", target_os = "linux"))]
    {
        prometheus::default_registry().register(Box::new(
            prometheus::process_collector::ProcessCollector::for_self()
        ))?;
    }
    
    Ok(())
}

#[cfg(feature = "metrics")]
pub fn gather_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    encoder.encode_to_string(&metric_families).unwrap()
}

#[cfg(not(feature = "metrics"))]
pub fn init_metrics() -> anyhow::Result<()> {
    Ok(())
}

#[cfg(not(feature = "metrics"))]
pub fn gather_metrics() -> String {
    String::new()
}