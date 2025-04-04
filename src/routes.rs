use actix_web::web;
use crate::handlers::{create_url, redirect, get_stats};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            // URL shortening endpoints
            .service(web::resource("/shorten")
                .route(web::post().to(create_url)))
            // Stats endpoints
            .service(web::resource("/stats/{short_code}")
                .route(web::get().to(get_stats)))
    )
    // Redirect endpoint
    .service(web::resource("/{short_code}")
        .route(web::get().to(redirect)));
}