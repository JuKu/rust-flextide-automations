//! Credentials API endpoints
//!
//! Provides REST API endpoints for managing credentials (API keys, tokens, etc.)

#[allow(unused_imports)] // delete, post, put are used in create_router() but compiler doesn't see macro usage
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use flextide_core::credentials::{
    list_credentials, get_credential, create_credential, update_credential, delete_credential,
    CredentialsManager,
};
use flextide_core::jwt::Claims;
use flextide_core::user::{user_belongs_to_organization, user_has_permission};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::AppState;

// Note: CredentialMetadataResponse was removed as it's not currently used
// The API directly converts CredentialMetadata to JSON in the endpoints

/// Create credential request
#[derive(Debug, Deserialize)]
pub struct CreateCredentialRequest {
    pub name: String,
    pub credential_type: String,
    pub data: Value,
}

/// Update credential request
#[derive(Debug, Deserialize)]
pub struct UpdateCredentialRequest {
    pub name: Option<String>,
    pub data: Option<Value>, // If None or empty, keep the old value
}

/// List all credentials for the current organization
///
/// GET /api/credentials
/// Returns a list of all credentials (without their values) for the authenticated user's organization
pub async fn list_credentials_endpoint(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Extension(org_uuid): Extension<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
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
        "can_see_all_credentials",
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
                "error": "User does not have permission to see credentials"
            })),
        ));
    }

    let credentials = list_credentials(&state.db_pool, &org_uuid, &claims.user_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list credentials: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to list credentials" })),
            )
        })?;

    let credentials_json: Vec<Value> = credentials
        .into_iter()
        .map(|c| {
            json!({
                "uuid": c.uuid,
                "organization_uuid": c.organization_uuid,
                "name": c.name,
                "credential_type": c.credential_type,
                "creator_user_uuid": c.creator_user_uuid,
                "created_at": c.created_at.to_rfc3339(),
                "updated_at": c.updated_at.map(|dt| dt.to_rfc3339()),
            })
        })
        .collect();

    Ok(Json(json!(credentials_json)))
}

/// Get a credential by UUID (with decrypted data)
///
/// GET /api/credentials/{uuid}
/// Returns a specific credential with decrypted data
pub async fn get_credential_endpoint(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Extension(org_uuid): Extension<String>,
    Path(credential_uuid): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
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
        "can_see_all_credentials",
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
                "error": "User does not have permission to see credentials"
            })),
        ));
    }

    let manager = CredentialsManager::new().map_err(|e| {
        tracing::error!("Failed to create credentials manager: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to initialize credentials manager" })),
        )
    })?;

    let credential = get_credential(
        &state.db_pool,
        &manager,
        &credential_uuid,
        &org_uuid,
        &claims.user_uuid,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to get credential: {}", e);
        match e {
            flextide_core::credentials::CredentialsError::CredentialNotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Credential not found" })),
            ),
            flextide_core::credentials::CredentialsError::PermissionDenied => (
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "Permission denied" })),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to get credential" })),
            ),
        }
    })?;

    Ok(Json(json!({
        "uuid": credential.uuid,
        "organization_uuid": credential.organization_uuid,
        "name": credential.name,
        "credential_type": credential.credential_type,
        "data": credential.data,
        "creator_user_uuid": credential.creator_user_uuid,
        "created_at": credential.created_at.to_rfc3339(),
        "updated_at": credential.updated_at.map(|dt| dt.to_rfc3339()),
    })))
}

/// Create a new credential
///
/// POST /api/credentials
/// Creates a new credential for the authenticated user's organization
pub async fn create_credential_endpoint(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Extension(org_uuid): Extension<String>,
    Json(payload): Json<CreateCredentialRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validate input
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Name is required" })),
        ));
    }

    if payload.credential_type.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Credential type is required" })),
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

    // Check permission (create permission - we'll use can_edit_credentials for now)
    let has_permission = user_has_permission(
        &state.db_pool,
        &claims.user_uuid,
        &org_uuid,
        "can_edit_credentials",
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
                "error": "User does not have permission to create credentials"
            })),
        ));
    }

    let manager = CredentialsManager::new().map_err(|e| {
        tracing::error!("Failed to create credentials manager: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to initialize credentials manager" })),
        )
    })?;

    let credential_uuid = create_credential(
        &state.db_pool,
        &manager,
        &org_uuid,
        &claims.user_uuid,
        &payload.name,
        &payload.credential_type,
        &payload.data,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to create credential: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to create credential" })),
        )
    })?;

    Ok(Json(json!({
        "uuid": credential_uuid,
        "message": "Credential created successfully"
    })))
}

/// Update an existing credential
///
/// PUT /api/credentials/{uuid}
/// Updates an existing credential. If data is not provided or is empty, keeps the old value.
pub async fn update_credential_endpoint(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Extension(org_uuid): Extension<String>,
    Path(credential_uuid): Path<String>,
    Json(payload): Json<UpdateCredentialRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
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
        "can_edit_credentials",
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
                "error": "User does not have permission to edit credentials"
            })),
        ));
    }

    let manager = CredentialsManager::new().map_err(|e| {
        tracing::error!("Failed to create credentials manager: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to initialize credentials manager" })),
        )
    })?;

    // If data is not provided or is empty, fetch the existing credential and use its data
    let data = if payload.data.is_none() || (payload.data.is_some() && payload.data.as_ref().unwrap().is_null()) {
        // Fetch existing credential to get current data
        let existing = get_credential(
            &state.db_pool,
            &manager,
            &credential_uuid,
            &org_uuid,
            &claims.user_uuid,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to get existing credential: {}", e);
            match e {
                flextide_core::credentials::CredentialsError::CredentialNotFound(_) => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Credential not found" })),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to get existing credential" })),
                ),
            }
        })?;
        existing.data
    } else {
        payload.data.unwrap()
    };

    update_credential(
        &state.db_pool,
        &manager,
        &credential_uuid,
        &org_uuid,
        &claims.user_uuid,
        payload.name.as_deref(),
        &data,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to update credential: {}", e);
        match e {
            flextide_core::credentials::CredentialsError::CredentialNotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Credential not found" })),
            ),
            flextide_core::credentials::CredentialsError::PermissionDenied => (
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "Permission denied" })),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to update credential" })),
            ),
        }
    })?;

    Ok(Json(json!({
        "message": "Credential updated successfully"
    })))
}

/// Delete a credential
///
/// DELETE /api/credentials/{uuid}
/// Deletes an existing credential
pub async fn delete_credential_endpoint(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Extension(org_uuid): Extension<String>,
    Path(credential_uuid): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
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
                "error": "User does not have permission to delete credentials"
            })),
        ));
    }

    delete_credential(
        &state.db_pool,
        &credential_uuid,
        &org_uuid,
        &claims.user_uuid,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete credential: {}", e);
        match e {
            flextide_core::credentials::CredentialsError::CredentialNotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Credential not found" })),
            ),
            flextide_core::credentials::CredentialsError::PermissionDenied => (
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "Permission denied" })),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to delete credential" })),
            ),
        }
    })?;

    Ok(Json(json!({
        "message": "Credential deleted successfully"
    })))
}

/// Create router for credentials endpoints
pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/credentials", get(list_credentials_endpoint).post(create_credential_endpoint))
        .route("/credentials/{uuid}", get(get_credential_endpoint).put(update_credential_endpoint).delete(delete_credential_endpoint))
}

