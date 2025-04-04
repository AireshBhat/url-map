use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use tracing::info;
use tracing_subscriber::EnvFilter;
use std::sync::Arc;

mod routes;
mod handlers;
mod services;
mod storage;
mod models;
mod errors;

use services::UrlService;
use storage::{MemoryStorage, StorageConfig};

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("Starting URL Shortener Service...");

    // Create storage layer
    let storage = Arc::new(MemoryStorage::new(StorageConfig::default()));
    
    // Create URL service with storage
    let url_service = web::Data::new(UrlService::new(storage));

    HttpServer::new(move || {
        App::new()
            // Add URL service to application state
            .app_data(url_service.clone())
            // Add logging middleware
            .wrap(tracing_actix_web::TracingLogger::default())
            // Add compression middleware
            .wrap(actix_web::middleware::Compress::default())
            // Add health check endpoint
            .route("/health", web::get().to(health_check))
            // Configure API routes
            .configure(routes::configure_routes)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
