use super::*;
use crate::errors::UrlShortenerErrorType;
use crate::storage::{MemoryStorage, StorageConfig};
use std::sync::Arc;

async fn create_test_service() -> UrlService {
    let storage = Arc::new(MemoryStorage::new(StorageConfig::default()));
    UrlService::new(storage)
}

#[tokio::test]
async fn test_create_short_url_success() {
    let service = create_test_service().await;
    let result = service.create_short_url("https://example.com".to_string()).await;
    
    assert!(result.is_ok());
    let shortened_url = result.unwrap();
    assert!(!shortened_url.short_code.is_empty());
    assert_eq!(shortened_url.original_url, "https://example.com/");
    assert_eq!(shortened_url.visits, 0);
}

#[tokio::test]
async fn test_create_short_url_invalid() {
    let service = create_test_service().await;
    let result = service.create_short_url("not-a-url".to_string()).await;
    
    assert!(result.is_err());
    match result.unwrap_err().error_type {
        UrlShortenerErrorType::InvalidUrl(_) => (),
        error_type => panic!("Expected InvalidUrl error, got {:?}", error_type),
    }
}

#[tokio::test]
async fn test_create_short_url_too_long() {
    let service = create_test_service().await;
    let long_url = "https://example.com/".repeat(1025); // 2048+ characters
    let result = service.create_short_url(long_url).await;
    
    assert!(result.is_err());
    match result.unwrap_err().error_type {
        UrlShortenerErrorType::UrlTooLong(_) => (),
        error_type => panic!("Expected UrlTooLong error, got {:?}", error_type),
    }
}

#[tokio::test]
async fn test_get_original_url_success() {
    let service = create_test_service().await;
    let original_url = "https://example.com".to_string();
    let shortened_url = service.create_short_url(original_url.clone()).await.unwrap();
    
    let result = service.get_original_url(&shortened_url.short_code).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "https://example.com/");
}

#[tokio::test]
async fn test_get_original_url_not_found() {
    let service = create_test_service().await;
    let result = service.get_original_url("nonexistent").await;
    
    assert!(result.is_err());
    match result.unwrap_err().error_type {
        UrlShortenerErrorType::NotFound => (),
        error_type => panic!("Expected NotFound error, got {:?}", error_type),
    }
}

#[tokio::test]
async fn test_get_original_url_increments_visits() {
    let service = create_test_service().await;
    let shortened_url = service.create_short_url("https://example.com".to_string()).await.unwrap();
    
    // First visit
    let _ = service.get_original_url(&shortened_url.short_code).await.unwrap();
    let stats = service.get_url_stats(&shortened_url.short_code).await.unwrap();
    assert_eq!(stats.visits, 1);
    
    // Second visit
    let _ = service.get_original_url(&shortened_url.short_code).await.unwrap();
    let stats = service.get_url_stats(&shortened_url.short_code).await.unwrap();
    assert_eq!(stats.visits, 2);
}

#[tokio::test]
async fn test_get_url_stats_success() {
    let service = create_test_service().await;
    let original_url = "https://example.com".to_string();
    let shortened_url = service.create_short_url(original_url.clone()).await.unwrap();
    
    let result = service.get_url_stats(&shortened_url.short_code).await;
    assert!(result.is_ok());
    let stats = result.unwrap();
    assert_eq!(stats.short_code, shortened_url.short_code);
    assert_eq!(stats.original_url, "https://example.com/");
    assert_eq!(stats.visits, 0);
}

#[tokio::test]
async fn test_get_url_stats_not_found() {
    let service = create_test_service().await;
    let result = service.get_url_stats("nonexistent").await;
    
    assert!(result.is_err());
    match result.unwrap_err().error_type {
        UrlShortenerErrorType::NotFound => (),
        error_type => panic!("Expected NotFound error, got {:?}", error_type),
    }
}
