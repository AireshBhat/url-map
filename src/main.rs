use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use tracing::info;
use tracing_subscriber::EnvFilter;

mod routes;
mod handlers;

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

    HttpServer::new(|| {
        App::new()
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
