use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use tracing::info;
use std::sync::Arc;

mod config;
mod errors;
mod handlers;
mod logging;
mod middleware;
mod models;
mod routes;
mod services;
mod storage;

use crate::config::Config;
use crate::logging::init_logging;
use crate::middleware::RequestLogger;
use crate::services::UrlService;
use crate::storage::PostgresStorage;

#[derive(serde::Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging with JSON formatting
    init_logging();

    // Load configuration
    let config = Config::from_env();
    let server_config = config.clone();

    // Initialize PostgreSQL storage
    let storage = Arc::new(
        PostgresStorage::new(config.to_storage_config())
            .await
            .expect("Failed to initialize PostgreSQL storage")
    );

    // Create URL service with PostgreSQL storage
    let url_service = web::Data::new(UrlService::new(storage));

    info!(
        host = %server_config.host,
        port = %server_config.port,
        "Starting server"
    );

    HttpServer::new(move || {
        App::new()
            // Add URL service to application state
            .app_data(url_service.clone())
            // Add our custom request logger
            .wrap(RequestLogger)
            // Add tracing integration
            .wrap(tracing_actix_web::TracingLogger::default())
            // Add compression middleware
            .wrap(actix_web::middleware::Compress::default())
            // Add health check endpoint
            .route("/health", web::get().to(health_check))
            // Configure API routes
            .configure(routes::configure_routes)
    })
    .bind((server_config.host, server_config.port))?
    .run()
    .await
}
