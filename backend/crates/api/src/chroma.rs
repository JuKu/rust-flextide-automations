//! Chroma Integration API endpoints

use axum::{
    extract::Extension,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use flextide_core::credentials::get_credentials_by_type;
use flextide_core::database::DatabasePool;
use flextide_core::jwt::Claims;
use integrations::chroma::{ChromaClient, ChromaCredentials};
use serde::Serialize;
use serde_json::{json, Value as JsonValue};

/// Create the Chroma API router
pub fn create_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/integrations/chroma/statistics", get(get_chroma_statistics))
        .route("/integrations/chroma/databases", get(list_chroma_databases))
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

/// Parse ChromaCredentials from JSON value
fn parse_chroma_credentials(data: &JsonValue) -> Result<ChromaCredentials, serde_json::Error> {
    serde_json::from_value(data.clone())
}


