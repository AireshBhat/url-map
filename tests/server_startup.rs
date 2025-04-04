use actix_web::{web, App, HttpResponse, test};

// Health check handler for testing
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[actix_rt::test]
async fn test_health_check() {
    // Create test app
    let app = test::init_service(
        App::new()
            .route("/health", web::get().to(health_check))
    ).await;

    // Create test request
    let req = test::TestRequest::get().uri("/health").to_request();

    // Send request and get response
    let resp = test::call_service(&app, req).await;

    // Check status code
    assert!(resp.status().is_success());

    // Parse response body
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

    // Verify response content
    assert_eq!(json["status"], "ok");
    assert_eq!(json["version"], env!("CARGO_PKG_VERSION"));
}
