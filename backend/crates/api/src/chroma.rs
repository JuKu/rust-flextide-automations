//! Chroma Integration API endpoints

// delete and put are used in route macros but compiler doesn't detect macro usage
#[allow(unused_imports)]
use axum::{
    extract::{Extension, Path, Query, State},
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
use integrations::chroma::{ChromaClient, ChromaCredentials, ChromaError, CreateCollectionRequest, UpdateCollectionRequest};
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
        .route("/integrations/chroma/collections", get(list_chroma_collections).post(create_chroma_collection))
        .route("/integrations/chroma/collections/{collection_id}", get(get_chroma_collection).put(update_chroma_collection).delete(delete_chroma_collection))
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

    let mut all_collections: Vec<ChromaCollectionInfo> = Vec::new();

    // Fetch collections from each database
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
                    for collection in collections {
                        all_collections.push(ChromaCollectionInfo {
                            id: collection.id,
                            name: collection.name,
                            database_uuid: cred.uuid.clone(),
                            database_name: chroma_creds.database_name.clone(),
                            tenant_name: chroma_creds.tenant_name.clone(),
                            document_count: 0, // TODO: Get actual document count if needed
                        });
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to list collections for database {}: {}", cred.name, e);
                }
            }
        }
    }

    Ok(Json(json!({ "collections": all_collections })))
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

/// Request to create a new Chroma collection
#[derive(Debug, Deserialize)]
pub struct CreateChromaCollectionRequest {
    pub database_uuid: String,
    pub name: String,
    #[serde(default)]
    pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
}

/// Request to update a Chroma collection
#[derive(Debug, Deserialize)]
pub struct UpdateChromaCollectionRequest {
    pub database_uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_metadata: Option<serde_json::Map<String, serde_json::Value>>,
}

/// Request to delete a Chroma collection
#[derive(Debug, Deserialize)]
pub struct DeleteChromaCollectionRequest {
    pub database_uuid: String,
}

/// Create a new Chroma collection
///
/// POST /api/integrations/chroma/collections
pub async fn create_chroma_collection(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Extension(org_uuid): Extension<String>,
    Json(payload): Json<CreateChromaCollectionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // Validate input
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Collection name is required" })),
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
        "integration_chroma_can_add_collection",
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
                "error": "User does not have permission to create Chroma collections"
            })),
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

    // Get the database credential
    let credential = get_credential(
        &state.db_pool,
        &manager,
        &payload.database_uuid,
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

    // Create collection request
    // Convert serde_json::Map to HashMap if metadata is provided
    let metadata = payload.metadata.map(|map| {
        map.into_iter().collect::<std::collections::HashMap<String, serde_json::Value>>()
    });
    
    let create_request = CreateCollectionRequest {
        name: payload.name.trim().to_string(),
        metadata,
        embedding_function: None, // Can be extended later if needed
    };

    // Create the collection
    let collection = ChromaClient::create_collection_v2_with_credentials(
        &chroma_creds,
        &chroma_creds.tenant_name,
        &chroma_creds.database_name,
        create_request,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to create Chroma collection: {}", e);
        
        // Provide user-friendly error messages for specific error types
        let (status_code, error_message) = match e {
            ChromaError::CollectionExists(msg) => {
                // Try to extract collection name from error message
                let collection_name = if let Some(start) = msg.find('[') {
                    if let Some(end) = msg[start..].find(']') {
                        &msg[start + 1..start + end]
                    } else {
                        &payload.name.trim()
                    }
                } else {
                    &payload.name.trim()
                };
                (
                    StatusCode::CONFLICT,
                    format!("Collection '{}' already exists in this database", collection_name),
                )
            }
            ChromaError::InvalidApiKey => (
                StatusCode::UNAUTHORIZED,
                "Invalid API key or authentication failed. Please check your credentials.".to_string(),
            ),
            ChromaError::ApiError(msg) => {
                // Check if it's a 403/401/404 error
                if msg.contains("403") || msg.contains("Forbidden") {
                    (
                        StatusCode::FORBIDDEN,
                        "Invalid API key or authentication failed. Please check your credentials.".to_string(),
                    )
                } else if msg.contains("401") || msg.contains("Unauthorized") {
                    (
                        StatusCode::UNAUTHORIZED,
                        "Authentication failed. Please check your API key.".to_string(),
                    )
                } else if msg.contains("404") || msg.contains("Not Found") {
                    (
                        StatusCode::NOT_FOUND,
                        "Database or tenant not found. Please check your configuration.".to_string(),
                    )
                } else {
                    (
                        StatusCode::BAD_REQUEST,
                        format!("Failed to create collection: {}", msg),
                    )
                }
            }
            ChromaError::HttpError(e) => {
                if e.is_timeout() {
                    (
                        StatusCode::REQUEST_TIMEOUT,
                        "Connection timeout. Please check your network connection and try again.".to_string(),
                    )
                } else if e.is_connect() {
                    (
                        StatusCode::BAD_GATEWAY,
                        "Unable to connect to Chroma server. Please check the base URL.".to_string(),
                    )
                } else {
                    (
                        StatusCode::BAD_REQUEST,
                        format!("Connection error: {}", e),
                    )
                }
            }
            _ => (
                StatusCode::BAD_REQUEST,
                format!("Failed to create collection: {}", e),
            ),
        };
        
        (status_code, Json(json!({ "error": error_message })))
    })?;

    // Emit event for Chroma collection created
    let event = Event::new(
        "integration_chroma_collection_created",
        EventPayload::new(json!({
            "entity_type": "chroma_collection",
            "entity_id": collection.id,
            "data": {
                "name": collection.name,
                "database_uuid": payload.database_uuid,
                "tenant_name": chroma_creds.tenant_name,
                "database_name": chroma_creds.database_name,
            }
        }))
    )
    .with_organization(&org_uuid)
    .with_user(&claims.user_uuid);

    state.event_dispatcher.emit(event).await;

    Ok(Json(json!({
        "id": collection.id,
        "name": collection.name,
        "message": "Chroma collection created successfully"
    })))
}

/// Get a single Chroma collection
///
/// GET /api/integrations/chroma/collections/{collection_id}?database_uuid={uuid}
#[derive(Debug, Deserialize)]
pub struct GetChromaCollectionQuery {
    pub database_uuid: String,
}

pub async fn get_chroma_collection(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Extension(org_uuid): Extension<String>,
    Path(collection_id): Path<String>,
    Query(query): Query<GetChromaCollectionQuery>,
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

    // Get the database credential
    let credential = get_credential(
        &state.db_pool,
        &manager,
        &query.database_uuid,
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

    // Get the collection - Chroma GET endpoint uses collection name, not ID
    // We need to find the collection by ID first by listing all collections
    let all_collections = ChromaClient::list_collections_v2_with_credentials(
        &chroma_creds,
        &chroma_creds.tenant_name,
        &chroma_creds.database_name,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to list Chroma collections: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("Failed to list collections: {}", e) })),
        )
    })?;

    // Find the collection by ID
    let collection = all_collections
        .into_iter()
        .find(|c| c.id == collection_id)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("Collection with ID {} not found", collection_id) })),
            )
        })?;

    // Chroma GET endpoint requires collection name, not ID
    // Call get_collection_v2_with_credentials with the name to get full details
    let full_collection = ChromaClient::get_collection_v2_with_credentials(
        &chroma_creds,
        &chroma_creds.tenant_name,
        &chroma_creds.database_name,
        &collection.name,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to get Chroma collection: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("Failed to get collection: {}", e) })),
        )
    })?;

    Ok(Json(json!({
        "id": full_collection.id,
        "name": full_collection.name,
        "metadata": full_collection.metadata,
        "database_uuid": query.database_uuid,
        "tenant_name": chroma_creds.tenant_name,
        "database_name": chroma_creds.database_name,
    })))
}

/// Update a Chroma collection
///
/// PUT /api/integrations/chroma/collections/{collection_id}
pub async fn update_chroma_collection(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Extension(org_uuid): Extension<String>,
    Path(collection_id): Path<String>,
    Json(payload): Json<UpdateChromaCollectionRequest>,
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

    // Check permission
    let has_permission = user_has_permission(
        &state.db_pool,
        &claims.user_uuid,
        &org_uuid,
        "integration_chroma_can_edit_collection",
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
                "error": "User does not have permission to edit Chroma collections"
            })),
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

    // Get the database credential
    let credential = get_credential(
        &state.db_pool,
        &manager,
        &payload.database_uuid,
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

    // First, find the collection by ID to get its current name
    // Chroma PUT endpoint uses collection ID, but we need the name to fetch it after update
    let all_collections = ChromaClient::list_collections_v2_with_credentials(
        &chroma_creds,
        &chroma_creds.tenant_name,
        &chroma_creds.database_name,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to list Chroma collections: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("Failed to list collections: {}", e) })),
        )
    })?;

    let existing_collection = all_collections
        .into_iter()
        .find(|c| c.id == collection_id)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("Collection with ID {} not found", collection_id) })),
            )
        })?;

    // Determine the collection name to use for update (may change if new_name is provided)
    let collection_name_for_update = payload.new_name
        .as_ref()
        .map(|n| n.trim().to_string())
        .unwrap_or_else(|| existing_collection.name.clone());

    // Convert metadata from serde_json::Map to HashMap if provided
    let new_metadata = payload.new_metadata.map(|map| {
        map.into_iter().collect::<std::collections::HashMap<String, serde_json::Value>>()
    });

    // Create update request
    let update_request = UpdateCollectionRequest {
        new_name: payload.new_name.map(|n| n.trim().to_string()),
        new_metadata,
        new_configuration: None, // Can be extended later if needed
    };

    // Update the collection (Chroma PUT uses collection ID in URL, but we pass name to the function)
    // Note: The function parameter is named 'name' but Chroma API may accept ID
    let updated_collection_opt = ChromaClient::update_collection_v2_with_credentials(
        &chroma_creds,
        &chroma_creds.tenant_name,
        &chroma_creds.database_name,
        &collection_id, // Using ID as per user's note that PUT uses ID
        update_request,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to update Chroma collection: {}", e);
        match e {
            ChromaError::InvalidApiKey => {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "error": "Invalid API key or authentication failed for the selected database." })),
                )
            }
            ChromaError::ApiError(msg) => {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": format!("Chroma API error: {}", msg) })),
                )
            }
            ChromaError::HttpError(msg) => {
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(json!({ "error": format!("Failed to connect to Chroma server: {}", msg) })),
                )
            }
            _ => {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": format!("Failed to update collection: {}", e) })),
                )
            }
        }
    })?;

    // If Chroma returned empty response, fetch the collection again using GET with the name
    let collection = if let Some(c) = updated_collection_opt {
        c
    } else {
        // Fetch the updated collection using GET endpoint (requires name, not ID)
        ChromaClient::get_collection_v2_with_credentials(
            &chroma_creds,
            &chroma_creds.tenant_name,
            &chroma_creds.database_name,
            &collection_name_for_update,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch updated Chroma collection: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("Failed to fetch updated collection: {}", e) })),
            )
        })?
    };

    // Emit event for Chroma collection updated
    let event = Event::new(
        "integration_chroma_collection_updated",
        EventPayload::new(json!({
            "entity_type": "chroma_collection",
            "entity_id": collection.id,
            "data": {
                "name": collection.name,
                "database_uuid": payload.database_uuid,
                "tenant_name": chroma_creds.tenant_name,
                "database_name": chroma_creds.database_name,
            }
        }))
    )
    .with_organization(&org_uuid)
    .with_user(&claims.user_uuid);

    state.event_dispatcher.emit(event).await;

    Ok(Json(json!({
        "id": collection.id,
        "name": collection.name,
        "metadata": collection.metadata,
        "message": "Chroma collection updated successfully"
    })))
}

/// Delete a Chroma collection
///
/// DELETE /api/integrations/chroma/collections/{collection_id}
pub async fn delete_chroma_collection(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Extension(org_uuid): Extension<String>,
    Path(collection_id): Path<String>,
    Json(payload): Json<DeleteChromaCollectionRequest>,
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

    // Check permission
    let has_permission = user_has_permission(
        &state.db_pool,
        &claims.user_uuid,
        &org_uuid,
        "integration_chroma_can_delete_collection",
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
                "error": "User does not have permission to delete Chroma collections"
            })),
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

    // Get the database credential
    let credential = get_credential(
        &state.db_pool,
        &manager,
        &payload.database_uuid,
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

    // Delete the collection (use collection_id, not name)
    ChromaClient::delete_collection_v2_with_credentials(
        &chroma_creds,
        &chroma_creds.tenant_name,
        &chroma_creds.database_name,
        &collection_id,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete Chroma collection: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("Failed to delete collection: {}", e) })),
        )
    })?;

    // Emit event for Chroma collection deleted
    let event = Event::new(
        "integration_chroma_collection_deleted",
        EventPayload::new(json!({
            "entity_type": "chroma_collection",
            "data": {
                "id": collection_id,
                "database_uuid": payload.database_uuid,
                "tenant_name": chroma_creds.tenant_name,
                "database_name": chroma_creds.database_name,
            }
        }))
    )
    .with_organization(&org_uuid)
    .with_user(&claims.user_uuid);

    state.event_dispatcher.emit(event).await;

    Ok(Json(json!({
        "message": "Chroma collection deleted successfully"
    })))
}

/// Parse ChromaCredentials from JSON value
fn parse_chroma_credentials(data: &JsonValue) -> Result<ChromaCredentials, serde_json::Error> {
    serde_json::from_value(data.clone())
}


