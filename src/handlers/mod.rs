use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::services::UrlService;
use crate::errors::UrlShortenerResult;

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
    service: web::Data<UrlService>,
) -> UrlShortenerResult<HttpResponse> {
    let shortened_url = service.create_short_url(request.original_url.clone()).await?;
    
    Ok(HttpResponse::Ok().json(CreateUrlResponse {
        short_url: shortened_url.short_code,
        original_url: shortened_url.original_url,
    }))
}

pub async fn redirect(
    short_code: web::Path<String>,
    service: web::Data<UrlService>,
) -> UrlShortenerResult<HttpResponse> {
    let original_url = service.get_original_url(&short_code).await?;
    
    Ok(HttpResponse::Found()
        .append_header(("Location", original_url))
        .finish())
}

pub async fn get_stats(
    short_code: web::Path<String>,
    service: web::Data<UrlService>,
) -> UrlShortenerResult<HttpResponse> {
    let stats = service.get_url_stats(&short_code).await?;
    
    Ok(HttpResponse::Ok().json(UrlStats {
        short_url: stats.short_code,
        original_url: stats.original_url,
        visits: stats.visits,
        created_at: stats.created_at.to_rfc3339(),
    }))
}

#[cfg(test)]
mod tests; 