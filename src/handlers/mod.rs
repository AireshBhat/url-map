use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use crate::services::{UrlService, ServiceError, ServiceResult};

// Request/Response models
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUrlRequest {
    pub original_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUrlResponse {
    pub short_url: String,
    pub original_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UrlStats {
    pub short_url: String,
    pub original_url: String,
    pub visits: u64,
    pub created_at: String,
}

// Handler functions
pub async fn create_url(
    request: web::Json<CreateUrlRequest>,
    service: web::Data<Mutex<UrlService>>,
) -> impl Responder {
    let mut service = service.lock().unwrap();
    match service.create_short_url(request.original_url.clone()) {
        Ok(shortened_url) => {
            HttpResponse::Ok().json(CreateUrlResponse {
                short_url: shortened_url.short_code,
                original_url: shortened_url.original_url,
            })
        }
        Err(ServiceError::InvalidUrl(_)) => {
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid URL provided"
            }))
        }
        Err(ServiceError::UrlTooLong) => {
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": "URL is too long"
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Internal server error: {}", e)
            }))
        }
    }
}

pub async fn redirect(
    short_code: web::Path<String>,
    service: web::Data<Mutex<UrlService>>,
) -> impl Responder {
    let mut service = service.lock().unwrap();
    match service.get_original_url(&short_code) {
        Ok(original_url) => HttpResponse::Found()
            .append_header(("Location", original_url))
            .finish(),
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("URL not found: {}", e)
        }))
    }
}

pub async fn get_stats(
    short_code: web::Path<String>,
    service: web::Data<Mutex<UrlService>>,
) -> impl Responder {
    let service = service.lock().unwrap();
    match service.get_url_stats(&short_code) {
        Ok(stats) => HttpResponse::Ok().json(UrlStats {
            short_url: stats.short_code,
            original_url: stats.original_url,
            visits: stats.visits,
            created_at: stats.created_at.to_rfc3339(),
        }),
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("URL not found: {}", e)
        }))
    }
}

#[cfg(test)]
mod tests; 