use axum_test::TestServer;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::{json, Value};
use uuid::Uuid;

mod common;
use api::Claims;

/// Helper function to create a JWT token for testing
fn create_test_token(email: &str, user_uuid: &str) -> String {
    use chrono::Utc;
    
    let now = Utc::now();
    let exp = (now + chrono::Duration::hours(24)).timestamp() as usize;
    let iat = now.timestamp() as usize;
    
    let claims = Claims {
        sub: email.to_string(),
        user_uuid: user_uuid.to_string(),
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


// Customer Creation Tests

#[tokio::test]
async fn test_create_customer_success() {
    let (app, org_uuid, user_uuid, email) = common::create_test_app_with_org().await;
    let server = TestServer::new(app).unwrap();
    
    let token = create_test_token(&email, &user_uuid);
    
    let response = server
        .post("/api/modules/crm/customers")
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", &org_uuid)
        .json(&json!({
            "first_name": "John",
            "last_name": "Doe",
            "email": "john.doe@example.com",
            "phone_number": "+1234567890",
            "company_name": "Example Corp"
        }))
        .await;
    
    response.assert_status_ok();
    
    let body: Value = response.json();
    assert!(body.get("uuid").is_some());
    assert_eq!(body.get("message").unwrap().as_str().unwrap(), "Customer created successfully");
}

#[tokio::test]
async fn test_create_customer_missing_required_fields() {
    let (app, org_uuid, user_uuid, email) = common::create_test_app_with_org().await;
    let server = TestServer::new(app).unwrap();
    
    let token = create_test_token(&email, &user_uuid);
    
    // Missing last_name - Axum returns 422 (Unprocessable Entity) for JSON deserialization failures
    let response = server
        .post("/api/modules/crm/customers")
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", &org_uuid)
        .json(&json!({
            "first_name": "John"
        }))
        .await;
    
    // Axum returns 422 when JSON deserialization fails (missing required field)
    assert_eq!(response.status_code(), 422);
}

#[tokio::test]
async fn test_create_customer_empty_first_name() {
    let (app, org_uuid, user_uuid, email) = common::create_test_app_with_org().await;
    let server = TestServer::new(app).unwrap();
    
    let token = create_test_token(&email, &user_uuid);
    
    let response = server
        .post("/api/modules/crm/customers")
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", &org_uuid)
        .json(&json!({
            "first_name": "   ",
            "last_name": "Doe"
        }))
        .await;
    
    response.assert_status_bad_request();
    
    let body: Value = response.json();
    assert_eq!(body.get("error").unwrap().as_str().unwrap(), "First name cannot be empty");
}

#[tokio::test]
async fn test_create_customer_without_auth() {
    let (app, org_uuid, _, _) = common::create_test_app_with_org().await;
    let server = TestServer::new(app).unwrap();
    
    let response = server
        .post("/api/modules/crm/customers")
        .add_header("X-Organization-UUID", &org_uuid)
        .json(&json!({
            "first_name": "John",
            "last_name": "Doe"
        }))
        .await;
    
    response.assert_status_unauthorized();
}

// Customer Search Tests

#[tokio::test]
async fn test_search_customers_success() {
    let (app, org_uuid, user_uuid, email) = common::create_test_app_with_org().await;
    let server = TestServer::new(app).unwrap();
    
    let token = create_test_token(&email, &user_uuid);
    
    // First, create a customer
    let create_response = server
        .post("/api/modules/crm/customers")
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", &org_uuid)
        .json(&json!({
            "first_name": "John",
            "last_name": "Doe",
            "email": "john.doe@example.com",
            "company_name": "Example Corp"
        }))
        .await;
    
    create_response.assert_status_ok();
    
    // Now search for the customer
    let search_response = server
        .get("/api/modules/crm/customers/search?q=John")
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", &org_uuid)
        .await;
    
    search_response.assert_status_ok();
    
    let body: Value = search_response.json();
    let customers = body.get("customers").unwrap().as_array().unwrap();
    assert_eq!(customers.len(), 1);
    assert_eq!(customers[0].get("name").unwrap().as_str().unwrap(), "John Doe");
}

#[tokio::test]
async fn test_search_customers_empty_query() {
    let (app, org_uuid, user_uuid, email) = common::create_test_app_with_org().await;
    let server = TestServer::new(app).unwrap();
    
    let token = create_test_token(&email, &user_uuid);
    
    let response = server
        .get("/api/modules/crm/customers/search?q=")
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", &org_uuid)
        .await;
    
    response.assert_status_bad_request();
    
    let body: Value = response.json();
    assert_eq!(body.get("error").unwrap().as_str().unwrap(), "Search query cannot be empty");
}

#[tokio::test]
async fn test_search_customers_no_results() {
    let (app, org_uuid, user_uuid, email) = common::create_test_app_with_org().await;
    let server = TestServer::new(app).unwrap();
    
    let token = create_test_token(&email, &user_uuid);
    
    let response = server
        .get("/api/modules/crm/customers/search?q=Nonexistent")
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", &org_uuid)
        .await;
    
    response.assert_status_ok();
    
    let body: Value = response.json();
    let customers = body.get("customers").unwrap().as_array().unwrap();
    assert_eq!(customers.len(), 0);
}

// Customer Deletion Tests

#[tokio::test]
async fn test_delete_customer_success() {
    let (app, org_uuid, user_uuid, email) = common::create_test_app_with_org().await;
    let server = TestServer::new(app).unwrap();
    
    let token = create_test_token(&email, &user_uuid);
    
    // Create a customer
    let create_response = server
        .post("/api/modules/crm/customers")
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", &org_uuid)
        .json(&json!({
            "first_name": "John",
            "last_name": "Doe"
        }))
        .await;
    
    create_response.assert_status_ok();
    let body: Value = create_response.json();
    let customer_uuid = body.get("uuid").unwrap().as_str().unwrap();
    
    // Delete the customer
    let delete_response = server
        .delete(&format!("/api/modules/crm/customers/{}", customer_uuid))
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", &org_uuid)
        .await;
    
    delete_response.assert_status_ok();
    
    let body: Value = delete_response.json();
    assert_eq!(body.get("message").unwrap().as_str().unwrap(), "Customer deleted successfully");
}

#[tokio::test]
async fn test_delete_customer_not_found() {
    let (app, org_uuid, user_uuid, email) = common::create_test_app_with_org().await;
    let server = TestServer::new(app).unwrap();
    
    let token = create_test_token(&email, &user_uuid);
    
    let fake_uuid = Uuid::new_v4().to_string();
    
    let response = server
        .delete(&format!("/api/modules/crm/customers/{}", fake_uuid))
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", &org_uuid)
        .await;
    
    response.assert_status_not_found();
}

// Customer Notes Tests

#[tokio::test]
async fn test_add_customer_note_success() {
    let (app, org_uuid, user_uuid, email) = common::create_test_app_with_org().await;
    let server = TestServer::new(app).unwrap();
    
    let token = create_test_token(&email, &user_uuid);
    
    // Create a customer
    let create_response = server
        .post("/api/modules/crm/customers")
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", &org_uuid)
        .json(&json!({
            "first_name": "John",
            "last_name": "Doe"
        }))
        .await;
    
    create_response.assert_status_ok();
    let body: Value = create_response.json();
    let customer_uuid = body.get("uuid").unwrap().as_str().unwrap();
    
    // Add a note
    let note_response = server
        .post(&format!("/api/modules/crm/customers/{}/notes", customer_uuid))
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", &org_uuid)
        .json(&json!({
            "note_text": "Customer called to inquire about pricing",
            "visible_to_customer": false
        }))
        .await;
    
    note_response.assert_status_ok();
    
    let body: Value = note_response.json();
    assert!(body.get("uuid").is_some());
    assert_eq!(body.get("message").unwrap().as_str().unwrap(), "Note added successfully");
}

#[tokio::test]
async fn test_add_customer_note_invalid_text() {
    let (app, org_uuid, user_uuid, email) = common::create_test_app_with_org().await;
    let server = TestServer::new(app).unwrap();
    
    let token = create_test_token(&email, &user_uuid);
    
    // Create a customer
    let create_response = server
        .post("/api/modules/crm/customers")
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", &org_uuid)
        .json(&json!({
            "first_name": "John",
            "last_name": "Doe"
        }))
        .await;
    
    create_response.assert_status_ok();
    let body: Value = create_response.json();
    let customer_uuid = body.get("uuid").unwrap().as_str().unwrap();
    
    // Try to add a note with invalid text (too short)
    let note_response = server
        .post(&format!("/api/modules/crm/customers/{}/notes", customer_uuid))
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", &org_uuid)
        .json(&json!({
            "note_text": "X"
        }))
        .await;
    
    note_response.assert_status_internal_server_error();
}

#[tokio::test]
async fn test_delete_customer_note_success() {
    let (app, org_uuid, user_uuid, email) = common::create_test_app_with_org().await;
    let server = TestServer::new(app).unwrap();
    
    let token = create_test_token(&email, &user_uuid);
    
    // Create a customer
    let create_response = server
        .post("/api/modules/crm/customers")
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", &org_uuid)
        .json(&json!({
            "first_name": "John",
            "last_name": "Doe"
        }))
        .await;
    
    create_response.assert_status_ok();
    let body: Value = create_response.json();
    let customer_uuid = body.get("uuid").unwrap().as_str().unwrap();
    
    // Add a note
    let note_response = server
        .post(&format!("/api/modules/crm/customers/{}/notes", customer_uuid))
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", &org_uuid)
        .json(&json!({
            "note_text": "Test note"
        }))
        .await;
    
    note_response.assert_status_ok();
    let note_body: Value = note_response.json();
    let note_uuid = note_body.get("uuid").unwrap().as_str().unwrap();
    
    // Delete the note
    let delete_response = server
        .delete(&format!("/api/modules/crm/customers/{}/notes/{}", customer_uuid, note_uuid))
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", &org_uuid)
        .await;
    
    delete_response.assert_status_ok();
    
    let body: Value = delete_response.json();
    assert_eq!(body.get("message").unwrap().as_str().unwrap(), "Note deleted successfully");
}

// Customer Addresses Tests

#[tokio::test]
async fn test_add_customer_address_success() {
    let (app, org_uuid, user_uuid, email) = common::create_test_app_with_org().await;
    let server = TestServer::new(app).unwrap();
    
    let token = create_test_token(&email, &user_uuid);
    
    // Create a customer
    let create_response = server
        .post("/api/modules/crm/customers")
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", &org_uuid)
        .json(&json!({
            "first_name": "John",
            "last_name": "Doe"
        }))
        .await;
    
    create_response.assert_status_ok();
    let body: Value = create_response.json();
    let customer_uuid = body.get("uuid").unwrap().as_str().unwrap();
    
    // Add an address
    let address_response = server
        .post(&format!("/api/modules/crm/customers/{}/addresses", customer_uuid))
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", &org_uuid)
        .json(&json!({
            "address_type": "billing",
            "street": "123 Main St",
            "city": "New York",
            "state_province": "NY",
            "postal_code": "10001",
            "country": "USA",
            "is_primary": true
        }))
        .await;
    
    address_response.assert_status_ok();
    
    let body: Value = address_response.json();
    assert!(body.get("uuid").is_some());
    assert_eq!(body.get("message").unwrap().as_str().unwrap(), "Address added successfully");
}

#[tokio::test]
async fn test_add_customer_address_empty_type() {
    let (app, org_uuid, user_uuid, email) = common::create_test_app_with_org().await;
    let server = TestServer::new(app).unwrap();
    
    let token = create_test_token(&email, &user_uuid);
    
    // Create a customer
    let create_response = server
        .post("/api/modules/crm/customers")
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", &org_uuid)
        .json(&json!({
            "first_name": "John",
            "last_name": "Doe"
        }))
        .await;
    
    create_response.assert_status_ok();
    let body: Value = create_response.json();
    let customer_uuid = body.get("uuid").unwrap().as_str().unwrap();
    
    // Try to add an address with empty type
    let address_response = server
        .post(&format!("/api/modules/crm/customers/{}/addresses", customer_uuid))
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", &org_uuid)
        .json(&json!({
            "address_type": "   "
        }))
        .await;
    
    address_response.assert_status_internal_server_error();
}

#[tokio::test]
async fn test_delete_customer_address_success() {
    let (app, org_uuid, user_uuid, email) = common::create_test_app_with_org().await;
    let server = TestServer::new(app).unwrap();
    
    let token = create_test_token(&email, &user_uuid);
    
    // Create a customer
    let create_response = server
        .post("/api/modules/crm/customers")
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", &org_uuid)
        .json(&json!({
            "first_name": "John",
            "last_name": "Doe"
        }))
        .await;
    
    create_response.assert_status_ok();
    let body: Value = create_response.json();
    let customer_uuid = body.get("uuid").unwrap().as_str().unwrap();
    
    // Add an address
    let address_response = server
        .post(&format!("/api/modules/crm/customers/{}/addresses", customer_uuid))
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", &org_uuid)
        .json(&json!({
            "address_type": "billing",
            "street": "123 Main St"
        }))
        .await;
    
    address_response.assert_status_ok();
    let address_body: Value = address_response.json();
    let address_uuid = address_body.get("uuid").unwrap().as_str().unwrap();
    
    // Delete the address
    let delete_response = server
        .delete(&format!("/api/modules/crm/customers/{}/addresses/{}", customer_uuid, address_uuid))
        .add_header("Authorization", format!("Bearer {}", token))
        .add_header("X-Organization-UUID", &org_uuid)
        .await;
    
    delete_response.assert_status_ok();
    
    let body: Value = delete_response.json();
    assert_eq!(body.get("message").unwrap().as_str().unwrap(), "Address deleted successfully");
}

