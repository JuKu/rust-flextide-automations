use axum_test::TestServer;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde_json::{json, Value};

mod common;
use api::Claims;

#[tokio::test]
async fn test_login_success() {
    let app = common::create_test_app().await;
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/api/login")
        .json(&json!({
            "email": "admin@example.com",
            "password": "admin"
        }))
        .await;

    response.assert_status_ok();
    
    let body: Value = response.json();
    assert!(body.get("token").is_some());
    assert_eq!(body.get("email").unwrap(), "admin@example.com");
    
    // Validate JWT token
    let token = body.get("token").unwrap().as_str().unwrap();
    let jwt_secret = "test-secret-key";
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    );
    assert!(token_data.is_ok());
    let claims = token_data.unwrap().claims;
    assert_eq!(claims.sub, "admin@example.com");
}

#[tokio::test]
async fn test_login_invalid_email() {
    let app = common::create_test_app().await;
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/api/login")
        .json(&json!({
            "email": "wrong@example.com",
            "password": "admin"
        }))
        .await;

    response.assert_status_unauthorized();
    
    let body: Value = response.json();
    assert_eq!(body.get("error").unwrap().as_str().unwrap(), "Invalid email or password");
}

#[tokio::test]
async fn test_login_invalid_password() {
    let app = common::create_test_app().await;
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/api/login")
        .json(&json!({
            "email": "admin@example.com",
            "password": "wrongpassword"
        }))
        .await;

    response.assert_status_unauthorized();
    
    let body: Value = response.json();
    assert_eq!(body.get("error").unwrap().as_str().unwrap(), "Invalid email or password");
}

#[tokio::test]
async fn test_login_missing_fields() {
    let app = common::create_test_app().await;
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/api/login")
        .json(&json!({
            "email": "admin@example.com"
        }))
        .await;

    // Axum returns 422 (Unprocessable Entity) for JSON deserialization failures
    assert_eq!(response.status_code(), 422);
}

#[tokio::test]
async fn test_register_success() {
    let app = common::create_test_app().await;
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/api/register")
        .json(&json!({
            "email": "newuser@example.com",
            "password": "password123"
        }))
        .await;

    response.assert_status_ok();
    
    let body: Value = response.json();
    assert!(body.get("token").is_some());
    assert_eq!(body.get("email").unwrap(), "newuser@example.com");
    
    // Validate JWT token
    let token = body.get("token").unwrap().as_str().unwrap();
    let jwt_secret = "test-secret-key";
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    );
    assert!(token_data.is_ok());
    let claims = token_data.unwrap().claims;
    assert_eq!(claims.sub, "newuser@example.com");
}

#[tokio::test]
async fn test_register_missing_fields() {
    let app = common::create_test_app().await;
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/api/register")
        .json(&json!({
            "email": "newuser@example.com"
        }))
        .await;

    // Axum returns 422 (Unprocessable Entity) for JSON deserialization failures
    assert_eq!(response.status_code(), 422);
}

#[tokio::test]
async fn test_health_check() {
    let app = common::create_test_app().await;
    let server = TestServer::new(app).unwrap();

    let response = server.get("/api/health").await;

    response.assert_status_ok();
    
    let body: Value = response.json();
    assert_eq!(body.get("status").unwrap().as_str().unwrap(), "ok");
}

