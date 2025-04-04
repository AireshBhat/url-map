use super::*;

#[test]
fn test_create_short_url_success() {
    let mut service = UrlService::new();
    let result = service.create_short_url("https://example.com".to_string());
    
    assert!(result.is_ok());
    let shortened_url = result.unwrap();
    assert!(!shortened_url.short_code.is_empty());
    assert_eq!(shortened_url.original_url, "https://example.com/");
    assert_eq!(shortened_url.visits, 0);
}

#[test]
fn test_create_short_url_invalid() {
    let mut service = UrlService::new();
    let result = service.create_short_url("not-a-url".to_string());
    
    assert!(result.is_err());
    match result.unwrap_err() {
        ServiceError::InvalidUrl(_) => (),
        _ => panic!("Expected InvalidUrl error"),
    }
}

#[test]
fn test_create_short_url_too_long() {
    let mut service = UrlService::new();
    let long_url = "https://example.com/".repeat(1025); // 2048+ characters
    let result = service.create_short_url(long_url);
    
    assert!(result.is_err());
    match result.unwrap_err() {
        ServiceError::UrlTooLong => (),
        _ => panic!("Expected UrlTooLong error"),
    }
}

#[test]
fn test_get_original_url_success() {
    let mut service = UrlService::new();
    let original_url = "https://example.com".to_string();
    let shortened_url = service.create_short_url(original_url.clone()).unwrap();
    
    let result = service.get_original_url(&shortened_url.short_code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "https://example.com/");
}

#[test]
fn test_get_original_url_not_found() {
    let mut service = UrlService::new();
    let result = service.get_original_url("nonexistent");
    
    assert!(result.is_err());
    match result.unwrap_err() {
        ServiceError::Internal(_) => (),
        _ => panic!("Expected Internal error"),
    }
}

#[test]
fn test_get_original_url_increments_visits() {
    let mut service = UrlService::new();
    let shortened_url = service.create_short_url("https://example.com".to_string()).unwrap();
    
    // First visit
    let _ = service.get_original_url(&shortened_url.short_code).unwrap();
    let stats = service.get_url_stats(&shortened_url.short_code).unwrap();
    assert_eq!(stats.visits, 1);
    
    // Second visit
    let _ = service.get_original_url(&shortened_url.short_code).unwrap();
    let stats = service.get_url_stats(&shortened_url.short_code).unwrap();
    assert_eq!(stats.visits, 2);
}

#[test]
fn test_get_url_stats_success() {
    let mut service = UrlService::new();
    let original_url = "https://example.com".to_string();
    let shortened_url = service.create_short_url(original_url.clone()).unwrap();
    
    let result = service.get_url_stats(&shortened_url.short_code);
    assert!(result.is_ok());
    let stats = result.unwrap();
    assert_eq!(stats.short_code, shortened_url.short_code);
    assert_eq!(stats.original_url, "https://example.com/");
    assert_eq!(stats.visits, 0);
}

#[test]
fn test_get_url_stats_not_found() {
    let service = UrlService::new();
    let result = service.get_url_stats("nonexistent");
    
    assert!(result.is_err());
    match result.unwrap_err() {
        ServiceError::Internal(_) => (),
        _ => panic!("Expected Internal error"),
    }
}

#[test]
fn test_short_code_generation() {
    let service = UrlService::new();
    let code1 = service.generate_short_code();
    let code2 = service.generate_short_code();
    
    assert_eq!(code1.len(), 6);
    assert_eq!(code2.len(), 6);
    assert_ne!(code1, code2); // Very small chance of collision, but possible
} 