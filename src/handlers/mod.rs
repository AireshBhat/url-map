use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

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
    pub visits: i64,
    pub created_at: String,
}

// Handler functions
pub async fn create_url(
    request: web::Json<CreateUrlRequest>,
) -> impl Responder {
    // TODO: Implement URL creation logic
    HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Not implemented yet",
        "status": "in_progress"
    }))
}

pub async fn redirect(
    short_code: web::Path<String>,
) -> impl Responder {
    // TODO: Implement redirect logic
    HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Not implemented yet",
        "status": "in_progress"
    }))
}

pub async fn get_stats(
    short_code: web::Path<String>,
) -> impl Responder {
    // TODO: Implement stats retrieval logic
    HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Not implemented yet",
        "status": "in_progress"
    }))
} 