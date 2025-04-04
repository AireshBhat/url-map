use actix_web::{test, web, App};
use crate::services::UrlService;
use crate::storage::{MemoryStorage, StorageConfig};
use std::sync::Arc;
use super::*;

async fn create_test_service() -> web::Data<UrlService> {
    let storage = Arc::new(MemoryStorage::new(StorageConfig::default()));
    web::Data::new(UrlService::new(storage))
}

#[actix_rt::test]
async fn test_create_url_success() {
    // Setup
    let service = create_test_service().await;
    let app = test::init_service(
        App::new()
            .app_data(service.clone())
            .service(web::resource("/api/shorten").route(web::post().to(create_url)))
    ).await;

    // Test request
    let req = test::TestRequest::post()
        .uri("/api/shorten")
        .set_json(&CreateUrlRequest {
            original_url: "https://example.com".to_string(),
        })
        .to_request();

    // Send request
    let resp = test::call_service(&app, req).await;

    // Check response
    assert!(resp.status().is_success());
    let body: CreateUrlResponse = test::read_body_json(resp).await;
    assert!(!body.short_url.is_empty());
    assert_eq!(body.original_url, "https://example.com/");
}

#[actix_rt::test]
async fn test_create_url_invalid() {
    // Setup
    let service = create_test_service().await;
    let app = test::init_service(
        App::new()
            .app_data(service.clone())
            .service(web::resource("/api/shorten").route(web::post().to(create_url)))
    ).await;

    // Test request with invalid URL
    let req = test::TestRequest::post()
        .uri("/api/shorten")
        .set_json(&CreateUrlRequest {
            original_url: "not-a-url".to_string(),
        })
        .to_request();

    // Send request
    let resp = test::call_service(&app, req).await;

    // Check response
    assert_eq!(resp.status().as_u16(), 400);
}

#[actix_rt::test]
async fn test_redirect_success() {
    // Setup
    let service = create_test_service().await;
    let shortened_url = service.create_short_url("https://example.com".to_string()).await.unwrap();

    let app = test::init_service(
        App::new()
            .app_data(service.clone())
            .service(web::resource("/{short_code}").route(web::get().to(redirect)))
    ).await;

    // Test request
    let req = test::TestRequest::get()
        .uri(&format!("/{}", shortened_url.short_code))
        .to_request();

    // Send request
    let resp = test::call_service(&app, req).await;

    // Check response
    assert_eq!(resp.status().as_u16(), 302);
    assert_eq!(
        resp.headers().get("Location").unwrap().to_str().unwrap(),
        "https://example.com/"
    );
}

#[actix_rt::test]
async fn test_redirect_not_found() {
    // Setup
    let service = create_test_service().await;
    let app = test::init_service(
        App::new()
            .app_data(service.clone())
            .service(web::resource("/{short_code}").route(web::get().to(redirect)))
    ).await;

    // Test request with non-existent short code
    let req = test::TestRequest::get()
        .uri("/nonexistent")
        .to_request();

    // Send request
    let resp = test::call_service(&app, req).await;

    // Check response
    assert_eq!(resp.status().as_u16(), 404);
}

#[actix_rt::test]
async fn test_get_stats_success() {
    // Setup
    let service = create_test_service().await;
    let shortened_url = service.create_short_url("https://example.com".to_string()).await.unwrap();

    let app = test::init_service(
        App::new()
            .app_data(service.clone())
            .service(web::resource("/api/stats/{short_code}").route(web::get().to(get_stats)))
    ).await;

    // Test request
    let req = test::TestRequest::get()
        .uri(&format!("/api/stats/{}", shortened_url.short_code))
        .to_request();

    // Send request
    let resp = test::call_service(&app, req).await;

    // Check response
    assert!(resp.status().is_success());
    let body: UrlStats = test::read_body_json(resp).await;
    assert_eq!(body.short_url, shortened_url.short_code);
    assert_eq!(body.original_url, "https://example.com/");
    assert_eq!(body.visits, 0);
}

#[actix_rt::test]
async fn test_get_stats_not_found() {
    // Setup
    let service = create_test_service().await;
    let app = test::init_service(
        App::new()
            .app_data(service.clone())
            .service(web::resource("/api/stats/{short_code}").route(web::get().to(get_stats)))
    ).await;

    // Test request with non-existent short code
    let req = test::TestRequest::get()
        .uri("/api/stats/nonexistent")
        .to_request();

    // Send request
    let resp = test::call_service(&app, req).await;

    // Check response
    assert_eq!(resp.status().as_u16(), 404);
}

#[actix_rt::test]
async fn test_visit_count_increment() {
    // Setup
    let service = create_test_service().await;
    let shortened_url = service.create_short_url("https://example.com".to_string()).await.unwrap();

    let app = test::init_service(
        App::new()
            .app_data(service.clone())
            .service(web::resource("/{short_code}").route(web::get().to(redirect)))
            .service(web::resource("/api/stats/{short_code}").route(web::get().to(get_stats)))
    ).await;

    // Make a redirect request to increment visit count
    let redirect_req = test::TestRequest::get()
        .uri(&format!("/{}", shortened_url.short_code))
        .to_request();
    let _ = test::call_service(&app, redirect_req).await;

    // Check stats to verify visit count was incremented
    let stats_req = test::TestRequest::get()
        .uri(&format!("/api/stats/{}", shortened_url.short_code))
        .to_request();
    let stats_resp = test::call_service(&app, stats_req).await;
    let stats: UrlStats = test::read_body_json(stats_resp).await;
    assert_eq!(stats.visits, 1);
} 