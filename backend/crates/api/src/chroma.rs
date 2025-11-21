//! Chroma Integration API endpoints

// delete and put are used in route macros but compiler doesn't detect macro usage
#[allow(unused_imports)]
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get, post, put},
    Router,
};
use flextide_core::credentials::{get_credentials_by_type, create_credential, get_credential, update_credential, delete_credential, CredentialsManager};
use flextide_core::database::DatabasePool;
use flextide_core::events::{Event, EventPayload};
use flextide_core::jwt::Claims;
use flextide_core::user::{user_belongs_to_organization, user_has_permission};
use integrations::chroma::{ChromaClient, ChromaCredentials, ChromaError};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use crate::AppState;

/// Create the Chroma API router
pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/integrations/chroma/statistics", get(get_chroma_statistics))
        .route("/integrations/chroma/databases", get(list_chroma_databases).post(create_chroma_database))
        .route("/integrations/chroma/databases/{uuid}", get(get_chroma_database).put(update_chroma_database).delete(delete_chroma_database))
        .route("/integrations/chroma/test-connection", post(test_chroma_connection))
        .route("/integrations/chroma/collections", get(list_chroma_collections))
}

#[derive(Debug, Serialize)]
pub struct ChromaStatistics {
    pub configured_databases: usize,
    pub total_collections: usize,
    pub total_documents: usize, // Mock for now
}

#[derive(Debug, Serialize)]
pub struct ChromaDatabaseInfo {
    pub uuid: String,
    pub name: String,
    pub base_url: String,
    pub tenant_name: String,
    pub database_name: String,
    pub secured_mode: bool,
}

#[derive(Debug, Serialize)]
pub struct ChromaCollectionInfo {
    pub id: String,
    pub name: String,
    pub database_uuid: String,
    pub database_name: String,
    pub tenant_name: String,
    pub document_count: usize, // Mock for now
}

/// Get Chroma integration statistics
///
/// GET /api/integrations/chroma/statistics
pub async fn get_chroma_statistics(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(_claims): Extension<Claims>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    use flextide_core::credentials::CredentialsManager;

    // Get credentials manager
    let manager = CredentialsManager::new().map_err(|e| {
        tracing::error!("Failed to initialize credentials manager: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to initialize credentials manager" })),
        )
    })?;

    // Get all Chroma database credentials
    let credentials = get_credentials_by_type(&pool, &manager, &org_uuid, "chroma_database")
        .await
        .map_err(|e| {
            tracing::error!("Failed to get Chroma credentials: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to get Chroma credentials" })),
            )
        })?;

    let configured_databases = credentials.len();

    // Count collections across all databases
    let mut total_collections = 0;
    for cred in &credentials {
        if let Ok(chroma_creds) = parse_chroma_credentials(&cred.data) {
            match ChromaClient::list_collections_v2_with_credentials(
                &chroma_creds,
                &chroma_creds.tenant_name,
                &chroma_creds.database_name,
            )
            .await
            {
                Ok(collections) => {
                    total_collections += collections.len();
                }
                Err(e) => {
                    tracing::warn!("Failed to list collections for database {}: {}", cred.name, e);
                }
            }
        }
    }

    let statistics = ChromaStatistics {
        configured_databases,
        total_collections,
        total_documents: 0, // Mock for now
    };

    Ok(Json(json!(statistics)))
}

/// List all configured Chroma databases
///
/// GET /api/integrations/chroma/databases
pub async fn list_chroma_databases(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(_claims): Extension<Claims>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    use flextide_core::credentials::CredentialsManager;

    // Get credentials manager
    let manager = CredentialsManager::new().map_err(|e| {
        tracing::error!("Failed to initialize credentials manager: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to initialize credentials manager" })),
        )
    })?;

    // Get all Chroma database credentials
    let credentials = get_credentials_by_type(&pool, &manager, &org_uuid, "chroma_database")
        .await
        .map_err(|e| {
            tracing::error!("Failed to get Chroma credentials: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to get Chroma credentials" })),
            )
        })?;

    let databases: Vec<ChromaDatabaseInfo> = credentials
        .into_iter()
        .filter_map(|cred| {
            parse_chroma_credentials(&cred.data).ok().map(|chroma_creds| {
                ChromaDatabaseInfo {
                    uuid: cred.uuid,
                    name: cred.name,
                    base_url: chroma_creds.base_url,
                    tenant_name: chroma_creds.tenant_name,
                    database_name: chroma_creds.database_name,
                    secured_mode: chroma_creds.secured_mode,
                }
            })
        })
        .collect();

    Ok(Json(json!({ "databases": databases })))
}

/// List all Chroma collections accessible to this organization
///
/// GET /api/integrations/chroma/collections
pub async fn list_chroma_collections(
    Extension(_pool): Extension<DatabasePool>,
    Extension(_org_uuid): Extension<String>,
    Extension(_claims): Extension<Claims>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // Mock data for now - will be implemented later
    let collections = vec![
        ChromaCollectionInfo {
            id: "mock-collection-1".to_string(),
            name: "Sample Collection 1".to_string(),
            database_uuid: "mock-db-1".to_string(),
            database_name: "Default Database".to_string(),
            tenant_name: "default_tenant".to_string(),
            document_count: 42,
        },
        ChromaCollectionInfo {
            id: "mock-collection-2".to_string(),
            name: "Sample Collection 2".to_string(),
            database_uuid: "mock-db-1".to_string(),
            database_name: "Default Database".to_string(),
            tenant_name: "default_tenant".to_string(),
            document_count: 15,
        },
    ];

    Ok(Json(json!({ "collections": collections })))
}

/// Request to create a new Chroma database connection
#[derive(Debug, Deserialize)]
pub struct CreateChromaDatabaseRequest {
    pub name: String,
    pub credentials: ChromaCredentials,
}

/// Request to test a Chroma database connection
#[derive(Debug, Deserialize)]
pub struct TestChromaConnectionRequest {
    pub credentials: ChromaCredentials,
}

/// Test Chroma database connection
///
/// POST /api/integrations/chroma/test-connection
pub async fn test_chroma_connection(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Extension(org_uuid): Extension<String>,
    Json(payload): Json<TestChromaConnectionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(&state.db_pool, &claims.user_uuid, &org_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    if !belongs {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "User does not belong to this organization" })),
        ));
    }

    // Test the connection
    match ChromaClient::test_connection_with_credentials(&payload.credentials).await {
        Ok(_) => Ok(Json(json!({ "success": true, "message": "Connection test successful" }))),
        Err(e) => {
            tracing::error!("Chroma connection test failed: {}", e);
            
            // Provide user-friendly error messages
            let error_message = match e {
                ChromaError::InvalidApiKey => {
                    "Invalid API key or authentication failed. Please check your credentials.".to_string()
                }
                ChromaError::CollectionNotFound(msg) => {
                    // Try to parse JSON error response from Chroma
                    let parsed_msg = if msg.trim_start().starts_with('{') {
                        // Try to parse as JSON
                        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&msg) {
                            // Extract message field if available
                            if let Some(message) = json_value.get("message").and_then(|v| v.as_str()) {
                                message.to_string()
                            } else if let Some(error) = json_value.get("error").and_then(|v| v.as_str()) {
                                error.to_string()
                            } else {
                                msg
                            }
                        } else {
                            msg
                        }
                    } else {
                        msg
                    };
                    
                    // Check if it's a tenant not found error
                    if parsed_msg.contains("Tenant") && parsed_msg.contains("not found") {
                        // Try to extract tenant name from error message
                        if let Some(tenant_start) = parsed_msg.find('[') {
                            if let Some(tenant_end) = parsed_msg[tenant_start..].find(']') {
                                let tenant_name = &parsed_msg[tenant_start + 1..tenant_start + tenant_end];
                                format!("Tenant '{}' not found. Please check the tenant name.", tenant_name)
                            } else {
                                "Tenant not found. Please check the tenant name.".to_string()
                            }
                        } else {
                            "Tenant not found. Please check the tenant name.".to_string()
                        }
                    } else if parsed_msg.contains("Database") && parsed_msg.contains("not found") {
                        // Try to extract database name from error message
                        if let Some(db_start) = parsed_msg.find('[') {
                            if let Some(db_end) = parsed_msg[db_start..].find(']') {
                                let db_name = &parsed_msg[db_start + 1..db_start + db_end];
                                format!("Database '{}' not found. Please check the database name.", db_name)
                            } else {
                                "Database not found. Please check the database name.".to_string()
                            }
                        } else {
                            "Database not found. Please check the database name.".to_string()
                        }
                    } else {
                        format!("Connection test failed: {}", parsed_msg)
                    }
                }
                ChromaError::ApiError(msg) => {
                    // Check if it's a 403 Forbidden error
                    if msg.contains("403") || msg.contains("Forbidden") {
                        "Invalid API key or authentication failed. Please check your credentials.".to_string()
                    } else if msg.contains("401") || msg.contains("Unauthorized") {
                        "Authentication failed. Please check your API key.".to_string()
                    } else if msg.contains("404") || msg.contains("Not Found") {
                        "Chroma server not found. Please check the base URL.".to_string()
                    } else if msg.contains("Tenant") && msg.contains("not found") {
                        // Try to extract tenant name from error message
                        if let Some(tenant_start) = msg.find('[') {
                            if let Some(tenant_end) = msg[tenant_start..].find(']') {
                                let tenant_name = &msg[tenant_start + 1..tenant_start + tenant_end];
                                format!("Tenant '{}' not found. Please check the tenant name.", tenant_name)
                            } else {
                                "Tenant not found. Please check the tenant name.".to_string()
                            }
                        } else {
                            "Tenant not found. Please check the tenant name.".to_string()
                        }
                    } else if msg.contains("Connection") || msg.contains("network") || msg.contains("timeout") {
                        "Unable to connect to Chroma server. Please check the base URL and network connectivity.".to_string()
                    } else {
                        format!("Connection test failed: {}", msg)
                    }
                }
                ChromaError::HttpError(http_err) => {
                    if http_err.is_timeout() {
                        "Connection timeout. Please check the base URL and network connectivity.".to_string()
                    } else if http_err.is_connect() {
                        "Unable to connect to Chroma server. Please check the base URL.".to_string()
                    } else {
                        format!("Connection failed: {}", http_err)
                    }
                }
                _ => format!("Connection test failed: {}", e),
            };
            
            Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": error_message })),
            ))
        }
    }
}

/// Create a new Chroma database connection
///
/// POST /api/integrations/chroma/databases
pub async fn create_chroma_database(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Extension(org_uuid): Extension<String>,
    Json(payload): Json<CreateChromaDatabaseRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // Validate input
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Name is required" })),
        ));
    }

    // Sanity checks and normalization for credentials
    let mut credentials = payload.credentials.clone();

    // Normalize base_url: trim and remove trailing slash
    credentials.base_url = credentials.base_url.trim().to_string();
    if credentials.base_url.ends_with('/') {
        credentials.base_url.pop();
    }

    // Validate base_url is not empty
    if credentials.base_url.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Base URL is required" })),
        ));
    }

    // Validate base_url starts with http:// or https://
    if !credentials.base_url.starts_with("http://") && !credentials.base_url.starts_with("https://") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Base URL must start with http:// or https://" })),
        ));
    }

    // Trim and validate tenant_name
    credentials.tenant_name = credentials.tenant_name.trim().to_string();
    if credentials.tenant_name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Tenant name is required" })),
        ));
    }

    // Trim and validate database_name
    credentials.database_name = credentials.database_name.trim().to_string();
    if credentials.database_name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Database name is required" })),
        ));
    }

    // Validate auth_method is one of the allowed values
    let valid_auth_methods = ["token", "basic_auth", "none"];
    if !valid_auth_methods.contains(&credentials.auth_method.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("Invalid auth_method. Must be one of: {}", valid_auth_methods.join(", ")) })),
        ));
    }

    // Trim token_transport_header
    credentials.token_transport_header = credentials.token_transport_header.trim().to_string();
    if credentials.auth_method != "none" && credentials.token_transport_header.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Token transport header is required when authentication is enabled" })),
        ));
    }

    // Trim token_prefix
    credentials.token_prefix = credentials.token_prefix.trim().to_string();

    // Validate auth_token if auth_method is not "none"
    if credentials.auth_method != "none" && credentials.auth_token.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Auth token is required when authentication is enabled" })),
        ));
    }

    // Validate basic_auth format (must contain colon)
    if credentials.auth_method == "basic_auth" && !credentials.auth_token.contains(':') {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Auth token must be in format 'username:password' for Basic Auth" })),
        ));
    }

    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(&state.db_pool, &claims.user_uuid, &org_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    if !belongs {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "User does not belong to this organization" })),
        ));
    }

    // Check permission
    let has_permission = user_has_permission(
        &state.db_pool,
        &claims.user_uuid,
        &org_uuid,
        "integration_chroma_can_add_database",
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error checking permission: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Database error" })),
        )
    })?;

    if !has_permission {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "User does not have permission to add Chroma databases"
            })),
        ));
    }

    // Test the connection before saving (use normalized credentials)
    match ChromaClient::test_connection_with_credentials(&credentials).await {
        Ok(_) => {
            tracing::info!("Chroma connection test successful for database: {}", payload.name);
        }
        Err(e) => {
            tracing::error!("Chroma connection test failed for database {}: {}", payload.name, e);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("Connection test failed: {}", e) })),
            ));
        }
    }

    // Get credentials manager
    let manager = CredentialsManager::new().map_err(|e| {
        tracing::error!("Failed to initialize credentials manager: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to initialize credentials manager" })),
        )
    })?;

    // Convert normalized credentials to JSON
    let credentials_json = serde_json::to_value(&credentials).map_err(|e| {
        tracing::error!("Failed to serialize credentials: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to serialize credentials" })),
        )
    })?;

    // Create credential (use trimmed name)
    let credential_uuid = create_credential(
        &state.db_pool,
        &manager,
        &org_uuid,
        &claims.user_uuid,
        &payload.name.trim(),
        "chroma_database",
        &credentials_json,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to create credential: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to create credential: {}", e) })),
        )
    })?;

    // Emit event for Chroma database connection created
    let event = Event::new(
        "integration_chroma_database_created",
        EventPayload::new(json!({
            "entity_type": "chroma_database",
            "entity_id": credential_uuid,
            "data": {
                "name": payload.name.trim(),
                "base_url": credentials.base_url,
                "tenant_name": credentials.tenant_name,
                "database_name": credentials.database_name,
                "secured_mode": credentials.secured_mode,
                "auth_method": credentials.auth_method,
            }
        }))
    )
    .with_organization(&org_uuid)
    .with_user(&claims.user_uuid);

    state.event_dispatcher.emit(event).await;

    Ok(Json(json!({
        "uuid": credential_uuid,
        "message": "Chroma database connection created successfully"
    })))
}

/// Get a single Chroma database connection with full credentials
///
/// GET /api/integrations/chroma/databases/{uuid}
pub async fn get_chroma_database(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Extension(org_uuid): Extension<String>,
    Path(uuid): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(&state.db_pool, &claims.user_uuid, &org_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    if !belongs {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "User does not belong to this organization" })),
        ));
    }

    // Get credentials manager
    let manager = CredentialsManager::new().map_err(|e| {
        tracing::error!("Failed to initialize credentials manager: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to initialize credentials manager" })),
        )
    })?;

    // Get credential
    let credential = get_credential(
        &state.db_pool,
        &manager,
        &uuid,
        &org_uuid,
        &claims.user_uuid,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to get credential: {}", e);
        match e {
            flextide_core::credentials::CredentialsError::CredentialNotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Chroma database connection not found" })),
            ),
            flextide_core::credentials::CredentialsError::PermissionDenied => (
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "Permission denied" })),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to get Chroma database connection" })),
            ),
        }
    })?;

    // Verify it's a chroma_database credential
    if credential.credential_type != "chroma_database" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Credential is not a Chroma database connection" })),
        ));
    }

    // Parse credentials
    let chroma_creds = parse_chroma_credentials(&credential.data).map_err(|e| {
        tracing::error!("Failed to parse Chroma credentials: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to parse Chroma credentials" })),
        )
    })?;

    Ok(Json(json!({
        "uuid": credential.uuid,
        "name": credential.name,
        "credentials": chroma_creds,
    })))
}

/// Update a Chroma database connection
///
/// PUT /api/integrations/chroma/databases/{uuid}
#[derive(Debug, Deserialize)]
pub struct UpdateChromaDatabaseRequest {
    pub name: String,
    pub credentials: ChromaCredentials,
}

pub async fn update_chroma_database(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Extension(org_uuid): Extension<String>,
    Path(uuid): Path<String>,
    Json(payload): Json<UpdateChromaDatabaseRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // Validate input
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Name is required" })),
        ));
    }

    let mut credentials = payload.credentials;

    // Normalize base_url (trim and remove trailing slash)
    credentials.base_url = credentials.base_url.trim().to_string();
    if credentials.base_url.ends_with('/') {
        credentials.base_url.pop();
    }
    if credentials.base_url.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Base URL is required" })),
        ));
    }

    // Validate required fields
    if credentials.tenant_name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Tenant name is required" })),
        ));
    }
    credentials.tenant_name = credentials.tenant_name.trim().to_string();

    if credentials.database_name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Database name is required" })),
        ));
    }
    credentials.database_name = credentials.database_name.trim().to_string();

    // Validate auth_method is one of the allowed values
    let valid_auth_methods = ["token", "basic_auth", "none"];
    if !valid_auth_methods.contains(&credentials.auth_method.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("Invalid auth_method. Must be one of: {}", valid_auth_methods.join(", ")) })),
        ));
    }

    // Trim token_transport_header
    credentials.token_transport_header = credentials.token_transport_header.trim().to_string();
    if credentials.auth_method != "none" && credentials.token_transport_header.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Token transport header is required when authentication is enabled" })),
        ));
    }

    // Trim token_prefix
    credentials.token_prefix = credentials.token_prefix.trim().to_string();

    // Validate auth_token if auth_method is not "none"
    if credentials.auth_method != "none" && credentials.auth_token.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Auth token is required when authentication is enabled" })),
        ));
    }

    // Validate basic_auth format (must contain colon)
    if credentials.auth_method == "basic_auth" && !credentials.auth_token.contains(':') {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Auth token must be in format 'username:password' for Basic Auth" })),
        ));
    }

    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(&state.db_pool, &claims.user_uuid, &org_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    if !belongs {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "User does not belong to this organization" })),
        ));
    }

    // Check permission
    let has_permission = user_has_permission(
        &state.db_pool,
        &claims.user_uuid,
        &org_uuid,
        "integration_chroma_can_add_database",
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error checking permission: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Database error" })),
        )
    })?;

    if !has_permission {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "User does not have permission to edit Chroma databases"
            })),
        ));
    }

    // Test the connection before saving
    match ChromaClient::test_connection_with_credentials(&credentials).await {
        Ok(_) => {
            tracing::info!("Chroma connection test successful for database update: {}", payload.name);
        }
        Err(e) => {
            tracing::error!("Chroma connection test failed for database update {}: {}", payload.name, e);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("Connection test failed: {}", e) })),
            ));
        }
    }

    // Get credentials manager
    let manager = CredentialsManager::new().map_err(|e| {
        tracing::error!("Failed to initialize credentials manager: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to initialize credentials manager" })),
        )
    })?;

    // Convert normalized credentials to JSON
    let credentials_json = serde_json::to_value(&credentials).map_err(|e| {
        tracing::error!("Failed to serialize credentials: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to serialize credentials" })),
        )
    })?;

    // Update credential
    update_credential(
        &state.db_pool,
        &manager,
        &uuid,
        &org_uuid,
        &claims.user_uuid,
        Some(&payload.name.trim()),
        &credentials_json,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to update credential: {}", e);
        match e {
            flextide_core::credentials::CredentialsError::CredentialNotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Chroma database connection not found" })),
            ),
            flextide_core::credentials::CredentialsError::PermissionDenied => (
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "Permission denied" })),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to update credential: {}", e) })),
            ),
        }
    })?;

    // Emit event for Chroma database connection updated
    let event = Event::new(
        "integration_chroma_database_updated",
        EventPayload::new(json!({
            "entity_type": "chroma_database",
            "entity_id": uuid,
            "data": {
                "name": payload.name.trim(),
                "base_url": credentials.base_url,
                "tenant_name": credentials.tenant_name,
                "database_name": credentials.database_name,
                "secured_mode": credentials.secured_mode,
                "auth_method": credentials.auth_method,
            }
        }))
    )
    .with_organization(&org_uuid)
    .with_user(&claims.user_uuid);

    state.event_dispatcher.emit(event).await;

    Ok(Json(json!({
        "uuid": uuid,
        "message": "Chroma database connection updated successfully"
    })))
}

/// Delete a Chroma database connection
///
/// DELETE /api/integrations/chroma/databases/{uuid}
pub async fn delete_chroma_database(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Extension(org_uuid): Extension<String>,
    Path(uuid): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(&state.db_pool, &claims.user_uuid, &org_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    if !belongs {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "User does not belong to this organization" })),
        ));
    }

    // Check permission (use can_delete_credentials permission)
    let has_permission = user_has_permission(
        &state.db_pool,
        &claims.user_uuid,
        &org_uuid,
        "can_delete_credentials",
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error checking permission: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Database error" })),
        )
    })?;

    if !has_permission {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "User does not have permission to delete Chroma databases"
            })),
        ));
    }

    // Delete credential
    delete_credential(
        &state.db_pool,
        &uuid,
        &org_uuid,
        &claims.user_uuid,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete credential: {}", e);
        match e {
            flextide_core::credentials::CredentialsError::CredentialNotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Chroma database connection not found" })),
            ),
            flextide_core::credentials::CredentialsError::PermissionDenied => (
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "Permission denied" })),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to delete credential: {}", e) })),
            ),
        }
    })?;

    // Emit event for Chroma database connection deleted
    let event = Event::new(
        "integration_chroma_database_deleted",
        EventPayload::new(json!({
            "entity_type": "chroma_database",
            "entity_id": uuid,
        }))
    )
    .with_organization(&org_uuid)
    .with_user(&claims.user_uuid);

    state.event_dispatcher.emit(event).await;

    Ok(Json(json!({
        "message": "Chroma database connection deleted successfully"
    })))
}

/// Parse ChromaCredentials from JSON value
fn parse_chroma_credentials(data: &JsonValue) -> Result<ChromaCredentials, serde_json::Error> {
    serde_json::from_value(data.clone())
}


