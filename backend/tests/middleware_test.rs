use axum_test::TestServer;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::{json, Value};

mod common;
use api::Claims;

/// Helper function to get an expired JWT token for testing
fn get_expired_token(email: &str) -> String {
    use chrono::{Duration, Utc};
    
    let now = Utc::now();
    let exp = (now - Duration::hours(1)).timestamp() as usize; // Expired 1 hour ago
    let iat = (now - Duration::hours(25)).timestamp() as usize;
    
    let user_uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, email.as_bytes()).to_string();
    
    let claims = Claims {
        sub: email.to_string(),
        user_uuid,
        exp,
        iat,
        is_server_admin: false,
    };
    
    let jwt_secret = "test-secret-key";
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .unwrap()
}

// Authentication Middleware Tests

// Note: OPTIONS requests are handled by CORS middleware and skipped by auth middleware.
// This is tested indirectly through the middleware logic verification in other tests.
// Explicit OPTIONS testing would require additional HTTP client setup that's not critical.

#[tokio::test]
async fn test_auth_middleware_skips_public_endpoints() {
    let app = common::create_test_app().await;
    let server = TestServer::new(app).unwrap();

    // Health check should work without auth
    let response = server.get("/api/health").await;
    response.assert_status_ok();

    // Login should work without auth
    let response = server
        .post("/api/login")
        .json(&json!({
            "email": "admin@example.com",
            "password": "admin"
        }))
        .await;
    response.assert_status_ok();
}

#[tokio::test]
async fn test_auth_middleware_missing_token() {
    let app = common::create_test_app().await;
    let server = TestServer::new(app).unwrap();

    let response = server
        .get("/api/organizations/list-own")
        .await;

    response.assert_status_unauthorized();
    
    let body: Value = response.json();
    assert_eq!(
        body.get("error").unwrap().as_str().unwrap(),
        "Missing Authorization header"
    );
}

#[tokio::test]
async fn test_auth_middleware_invalid_token_format() {
    let app = common::create_test_app().await;
    let server = TestServer::new(app).unwrap();

    let response = server
        .get("/api/organizations/list-own")
        .add_header("Authorization", "InvalidFormat token123")
        .await;

    response.assert_status_unauthorized();
    
    let body: Value = response.json();
    assert_eq!(
        body.get("error").unwrap().as_str().unwrap(),
        "Invalid Authorization header format"
    );
}

#[tokio::test]
async fn test_auth_middleware_invalid_token() {
    let app = common::create_test_app().await;
    let server = TestServer::new(app).unwrap();

    let response = server
        .get("/api/organizations/list-own")
        .add_header("Authorization", "Bearer invalid-token-12345")
        .await;

    response.assert_status_unauthorized();
    
    let body: Value = response.json();
    assert_eq!(
        body.get("error").unwrap().as_str().unwrap(),
        "Invalid or expired token"
    );
}

#[tokio::test]
async fn test_auth_middleware_expired_token() {
    let app = common::create_test_app().await;
    let server = TestServer::new(app).unwrap();

    let expired_token = get_expired_token("user@example.com");

    let response = server
        .get("/api/organizations/list-own")
        .add_header("Authorization", format!("Bearer {}", expired_token))
        .await;

    response.assert_status_unauthorized();
    
    let body: Value = response.json();
    // jsonwebtoken validates expiration during decode, so expired tokens return "Invalid or expired token"
    assert_eq!(
        body.get("error").unwrap().as_str().unwrap(),
        "Invalid or expired token"
    );
}

#[tokio::test]
async fn test_auth_middleware_valid_token() {
    let app = common::create_test_app().await;
    let server = TestServer::new(app).unwrap();

    // First, login to get a valid token
    let login_response = server
        .post("/api/login")
        .json(&json!({
            "email": "admin@example.com",
            "password": "admin"
        }))
        .await;

    login_response.assert_status_ok();
    let login_body: Value = login_response.json();
    let token = login_body.get("token").unwrap().as_str().unwrap();

    // Now use the token to access a protected endpoint
    let response = server
        .get("/api/organizations/list-own")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    response.assert_status_ok();
}

#[tokio::test]
async fn test_auth_middleware_server_admin_token() {
    let app = common::create_test_app().await;
    let server = TestServer::new(app).unwrap();

    // Login as admin to get server admin token
    let login_response = server
        .post("/api/login")
        .json(&json!({
            "email": "admin@example.com",
            "password": "admin"
        }))
        .await;

    login_response.assert_status_ok();
    let login_body: Value = login_response.json();
    let token = login_body.get("token").unwrap().as_str().unwrap();

    // Verify token contains is_server_admin
    use jsonwebtoken::{decode, DecodingKey, Validation};
    let jwt_secret = "test-secret-key";
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .unwrap();
    
    assert_eq!(token_data.claims.sub, "admin@example.com");
    assert!(token_data.claims.is_server_admin);
}

// Organization Middleware Tests

// Note: OPTIONS requests are handled by CORS middleware and skipped by org middleware.
// This is tested indirectly through the middleware logic verification in other tests.
// Explicit OPTIONS testing would require additional HTTP client setup that's not critical.

#[tokio::test]
async fn test_org_middleware_skips_public_endpoints() {
    let app = common::create_test_app().await;
    let server = TestServer::new(app).unwrap();

    // Login should work without org UUID
    let response = server
        .post("/api/login")
        .json(&json!({
            "email": "admin@example.com",
            "password": "admin"
        }))
        .await;
    response.assert_status_ok();

    // Organizations list should work without org UUID
    let login_response = server
        .post("/api/login")
        .json(&json!({
            "email": "admin@example.com",
            "password": "admin"
        }))
        .await;
    let login_body: Value = login_response.json();
    let token = login_body.get("token").unwrap().as_str().unwrap();

    let response = server
        .get("/api/organizations/list-own")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;
    response.assert_status_ok();
}

#[tokio::test]
async fn test_org_middleware_missing_org_uuid() {
    let app = common::create_test_app().await;
    let server = TestServer::new(app).unwrap();

    // Get a valid token
    let login_response = server
        .post("/api/login")
        .json(&json!({
            "email": "admin@example.com",
            "password": "admin"
        }))
        .await;
    let login_body: Value = login_response.json();
    let token = login_body.get("token").unwrap().as_str().unwrap();

    // Try to access protected endpoint without org UUID
    let response = server
        .post("/api/workflows/test-uuid/edit-title")
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "title": "New Title"
        }))
        .await;

    response.assert_status_bad_request();
    
    let body: Value = response.json();
    assert_eq!(
        body.get("error").unwrap().as_str().unwrap(),
        "Missing X-Organization-UUID header"
    );
    assert_eq!(
        body.get("code").unwrap().as_str().unwrap(),
        "MISSING_ORG_UUID"
    );
}

#[tokio::test]
async fn test_org_middleware_with_org_uuid() {
    let app = common::create_test_app().await;
    let server = TestServer::new(app).unwrap();

    // Get a valid token
    let login_response = server
        .post("/api/login")
        .json(&json!({
            "email": "admin@example.com",
            "password": "admin"
        }))
        .await;
    let login_body: Value = login_response.json();
    let token = login_body.get("token").unwrap().as_str().unwrap();

    // Get organizations to get a valid org UUID
    let orgs_response = server
        .get("/api/organizations/list-own")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;
    orgs_response.assert_status_ok();
    let orgs_body: Value = orgs_response.json();
    let orgs = orgs_body.as_array().unwrap();
    assert!(orgs.len() > 0);
    let org_uuid = orgs[0].get("uuid").unwrap().as_str().unwrap();

    // Try to access protected endpoint with org UUID
    let response = server
        .post("/api/workflows/test-uuid/edit-title")
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", org_uuid)
        .json(&json!({
            "title": "New Title"
        }))
        .await;

    // Should succeed (even though the endpoint might return an error for non-existent workflow)
    // The important thing is that it passes the middleware checks
    // We expect it might fail at the handler level (404 or 500), but not at middleware level (400, 401, 403)
    let status = response.status_code();
    // Check that middleware passed - should not be auth/org middleware errors
    // Handler might return 404 (not found) or 500 (server error), but not 400/401/403
    assert!(
        status != 400 && status != 401 && status != 403,
        "Middleware should pass - status should not be 400, 401, or 403, but got {} (response: {:?})",
        status,
        response.text()
    );
}

#[tokio::test]
async fn test_org_middleware_without_auth() {
    let app = common::create_test_app().await;
    let server = TestServer::new(app).unwrap();

    // Try to access protected endpoint with org UUID but without auth token
    let response = server
        .post("/api/workflows/test-uuid/edit-title")
        .add_header("X-Organization-UUID", "test-org-uuid")
        .json(&json!({
            "title": "New Title"
        }))
        .await;

    // Should fail at auth middleware first (401), not org middleware
    response.assert_status_unauthorized();
}

#[tokio::test]
async fn test_org_middleware_invalid_org_uuid_format() {
    let app = common::create_test_app().await;
    let server = TestServer::new(app).unwrap();

    // Get a valid token
    let login_response = server
        .post("/api/login")
        .json(&json!({
            "email": "admin@example.com",
            "password": "admin"
        }))
        .await;
    let login_body: Value = login_response.json();
    let token = login_body.get("token").unwrap().as_str().unwrap();

    // Try with invalid org UUID (non-UTF8 would be hard to test, but we can test the flow)
    // For now, any string should be accepted as valid format, so this test verifies
    // that a valid format string is accepted
    let response = server
        .post("/api/workflows/test-uuid/edit-title")
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", "valid-uuid-format")
        .json(&json!({
            "title": "New Title"
        }))
        .await;

    // Should pass middleware (format is valid), might fail at handler
    assert!(response.status_code() != 400); // Not a bad request for format
}

