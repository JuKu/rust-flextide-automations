//! Docs API endpoints
//!
//! Provides REST API endpoints for managing documentation and related resources.

use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get, post, put},
    Router,
};
use serde::Deserialize;
use flextide_core::database::DatabasePool;
use flextide_core::events::EventDispatcher;
use flextide_core::jwt::Claims;
use serde_json::{json, Value as JsonValue};

use crate::area::{
    create_area, delete_area, load_area_by_uuid, update_area, CreateDocsAreaRequest,
    DocsAreaDatabaseError, UpdateDocsAreaRequest,
};
use crate::folder::{
    create_folder, delete_folder, list_folders, reorder_folder, update_folder_name,
    CreateDocsFolderRequest, DocsFolderDatabaseError, UpdateDocsFolderRequest,
};
use crate::page::{list_pages, DocsPageDatabaseError};
use flextide_core::user::user_belongs_to_organization;

/// Create the API router for Docs endpoints
pub fn create_api_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/modules/docs/documents", get(list_documents))
        .route("/modules/docs/areas", post(create_area_endpoint))
        .route(
            "/modules/docs/areas/{uuid}",
            get(get_area_endpoint)
                .put(update_area_endpoint)
                .delete(delete_area_endpoint),
        )
        .route("/modules/docs/areas/{area_uuid}/folders", get(list_folders_endpoint))
        .route("/modules/docs/areas/{area_uuid}/folders", post(create_folder_endpoint))
        .route("/modules/docs/areas/{area_uuid}/pages", get(list_pages_endpoint))
        .route("/modules/docs/folders/{uuid}", delete(delete_folder_endpoint))
        .route("/modules/docs/folders/{uuid}/name", put(update_folder_name_endpoint))
        .route(
            "/modules/docs/folders/{uuid}/sort-order",
            put(reorder_folder_endpoint),
        )
}

async fn list_documents(
    Extension(_pool): Extension<DatabasePool>,
    Extension(_org_uuid): Extension<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // TODO: Implement document listing
    Ok(Json(json!({
        "documents": [],
        "total": 0
    })))
}

/// Query parameters for listing pages
#[derive(Debug, Deserialize)]
struct ListPagesQuery {
    folder_uuid: Option<String>,
}

/// List pages for an area
///
/// GET /api/modules/docs/areas/{area_uuid}/pages?folder_uuid={uuid}
pub async fn list_pages_endpoint(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Path(area_uuid): Path<String>,
    Query(query): Query<ListPagesQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(&pool, &claims.user_uuid, &org_uuid)
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

    // List pages (permission checks are done inside list_pages)
    let pages = list_pages(
        &pool,
        &org_uuid,
        &area_uuid,
        query.folder_uuid.as_deref(),
        &claims.user_uuid,
    )
    .await
    .map_err(|e| {
        tracing::error!("Error listing pages: {}", e);
        match e {
            DocsPageDatabaseError::UserNotInOrganization => (
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "User does not belong to this organization" })),
            ),
            DocsPageDatabaseError::PermissionDenied => (
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "User does not have permission to view pages in this area" })),
            ),
            DocsPageDatabaseError::AreaNotFound => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Area not found" })),
            ),
            DocsPageDatabaseError::AreaNotInOrganization => (
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "Area does not belong to this organization" })),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to list pages" })),
            ),
        }
    })?;

    Ok(Json(json!({
        "pages": pages,
        "total": pages.len()
    })))
}

/// Create a new area
///
/// POST /api/modules/docs/areas
pub async fn create_area_endpoint(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Extension(dispatcher): Extension<EventDispatcher>,
    Json(request): Json<CreateDocsAreaRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // Sanity checks
    if request.short_name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Short name cannot be empty" })),
        ));
    }

    // Validate short name length
    if request.short_name.len() > 255 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Short name cannot exceed 255 characters" })),
        ));
    }

    // Create area (permission checks are done inside create_area)
    let area_uuid = create_area(&pool, &org_uuid, &claims.user_uuid, request, Some(&dispatcher))
        .await
        .map_err(|e| {
            tracing::error!("Error creating area: {}", e);
            match e {
                DocsAreaDatabaseError::UserNotInOrganization => (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "User does not belong to this organization" })),
                ),
                DocsAreaDatabaseError::PermissionDenied => (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "User does not have permission to create areas" })),
                ),
                DocsAreaDatabaseError::EmptyShortName => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "Short name cannot be empty" })),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to create area" })),
                ),
            }
        })?;

    // Load the created area to return
    let area = load_area_by_uuid(&pool, &area_uuid).await.map_err(|e| {
        tracing::error!("Error loading created area: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Area created but failed to load" })),
        )
    })?;

    Ok(Json(json!({
        "uuid": area_uuid,
        "area": area,
        "message": "Area created successfully"
    })))
}

/// Get an area by UUID
///
/// GET /api/modules/docs/areas/{uuid}
pub async fn get_area_endpoint(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Path(area_uuid): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(&pool, &claims.user_uuid, &org_uuid)
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

    // Load area
    let area = load_area_by_uuid(&pool, &area_uuid).await.map_err(|e| {
        tracing::error!("Error loading area: {}", e);
        match e {
            DocsAreaDatabaseError::AreaNotFound => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Area not found" })),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to load area" })),
            ),
        }
    })?;

    // Verify area belongs to the organization
    if area.organization_uuid != org_uuid {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Area does not belong to this organization" })),
        ));
    }

    Ok(Json(json!({
        "area": area
    })))
}

/// Update an area
///
/// PUT /api/modules/docs/areas/{uuid}
pub async fn update_area_endpoint(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Extension(dispatcher): Extension<EventDispatcher>,
    Path(area_uuid): Path<String>,
    Json(request): Json<UpdateDocsAreaRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // Sanity checks
    if let Some(ref short_name) = request.short_name {
        if short_name.trim().is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Short name cannot be empty" })),
            ));
        }

        if short_name.len() > 255 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Short name cannot exceed 255 characters" })),
            ));
        }
    }

    // Update area (permission checks are done inside update_area)
    update_area(&pool, &area_uuid, &org_uuid, &claims.user_uuid, request, Some(&dispatcher))
        .await
        .map_err(|e| {
            tracing::error!("Error updating area: {}", e);
            match e {
                DocsAreaDatabaseError::UserNotInOrganization => (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "User does not belong to this organization" })),
                ),
                DocsAreaDatabaseError::PermissionDenied => (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "User does not have permission to edit this area" })),
                ),
                DocsAreaDatabaseError::AreaNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Area not found" })),
                ),
                DocsAreaDatabaseError::AreaNotInOrganization => (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "Area does not belong to this organization" })),
                ),
                DocsAreaDatabaseError::EmptyShortName => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "Short name cannot be empty" })),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to update area" })),
                ),
            }
        })?;

    // Load the updated area to return
    let area = load_area_by_uuid(&pool, &area_uuid).await.map_err(|e| {
        tracing::error!("Error loading updated area: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Area updated but failed to load" })),
        )
    })?;

    Ok(Json(json!({
        "area": area,
        "message": "Area updated successfully"
    })))
}

/// Delete an area
///
/// DELETE /api/modules/docs/areas/{uuid}
pub async fn delete_area_endpoint(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Extension(dispatcher): Extension<EventDispatcher>,
    Path(area_uuid): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // Load area first to get data for response (before deletion)
    let area = load_area_by_uuid(&pool, &area_uuid).await.map_err(|e| {
        tracing::error!("Error loading area: {}", e);
        match e {
            DocsAreaDatabaseError::AreaNotFound => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Area not found" })),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to load area" })),
            ),
        }
    })?;

    // Verify area belongs to the organization
    if area.organization_uuid != org_uuid {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Area does not belong to this organization" })),
        ));
    }

    // Delete area (permission checks are done inside delete_area)
    delete_area(&pool, &area_uuid, &org_uuid, &claims.user_uuid, Some(&dispatcher))
        .await
        .map_err(|e| {
            tracing::error!("Error deleting area: {}", e);
            match e {
                DocsAreaDatabaseError::UserNotInOrganization => (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "User does not belong to this organization" })),
                ),
                DocsAreaDatabaseError::PermissionDenied => (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "User does not have permission to delete this area or area is not deletable" })),
                ),
                DocsAreaDatabaseError::AreaNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Area not found" })),
                ),
                DocsAreaDatabaseError::AreaNotInOrganization => (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "Area does not belong to this organization" })),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to delete area" })),
                ),
            }
        })?;

    Ok(Json(json!({
        "message": "Area deleted successfully"
    })))
}

/// Query parameters for listing folders
#[derive(Debug, Deserialize)]
struct ListFoldersQuery {
    parent_folder_uuid: Option<String>,
}

/// List folders for an area
///
/// GET /api/modules/docs/areas/{area_uuid}/folders?parent_folder_uuid={uuid}
pub async fn list_folders_endpoint(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Path(area_uuid): Path<String>,
    Query(query): Query<ListFoldersQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(&pool, &claims.user_uuid, &org_uuid)
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

    // Load area to verify it belongs to the organization
    let area = load_area_by_uuid(&pool, &area_uuid).await.map_err(|e| {
        tracing::error!("Error loading area: {}", e);
        match e {
            DocsAreaDatabaseError::AreaNotFound => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Area not found" })),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to load area" })),
            ),
        }
    })?;

    if area.organization_uuid != org_uuid {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Area does not belong to this organization" })),
        ));
    }

    // List folders
    let folders = list_folders(
        &pool,
        &org_uuid,
        &area_uuid,
        query.parent_folder_uuid.as_deref(),
    )
    .await
    .map_err(|e| {
        tracing::error!("Error listing folders: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to list folders" })),
        )
    })?;

    Ok(Json(json!({
        "folders": folders
    })))
}

/// Create a new folder
///
/// POST /api/modules/docs/areas/{area_uuid}/folders
pub async fn create_folder_endpoint(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Extension(dispatcher): Extension<EventDispatcher>,
    Path(area_uuid): Path<String>,
    Json(mut request): Json<CreateDocsFolderRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // Set area_uuid from path
    request.area_uuid = area_uuid.clone();

    // Sanity checks
    if request.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Name cannot be empty" })),
        ));
    }

    // Validate name length
    if request.name.len() > 255 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Name cannot exceed 255 characters" })),
        ));
    }

    // Create folder (permission checks are done inside create_folder)
    let folder_uuid = create_folder(&pool, &org_uuid, &claims.user_uuid, request, Some(&dispatcher))
        .await
        .map_err(|e| {
            tracing::error!("Error creating folder: {}", e);
            match e {
                DocsFolderDatabaseError::UserNotInOrganization => (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "User does not belong to this organization" })),
                ),
                DocsFolderDatabaseError::PermissionDenied => (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "User does not have permission to create folders" })),
                ),
                DocsFolderDatabaseError::EmptyName => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "Name cannot be empty" })),
                ),
                DocsFolderDatabaseError::AreaNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Area not found" })),
                ),
                DocsFolderDatabaseError::AreaNotInOrganization => (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "Area does not belong to this organization" })),
                ),
                DocsFolderDatabaseError::FolderNotInOrganization => (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "Parent folder does not belong to this organization" })),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to create folder" })),
                ),
            }
        })?;

    Ok(Json(json!({
        "uuid": folder_uuid,
        "message": "Folder created successfully"
    })))
}

/// Delete a folder
///
/// DELETE /api/modules/docs/folders/{uuid}
pub async fn delete_folder_endpoint(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Extension(dispatcher): Extension<EventDispatcher>,
    Path(folder_uuid): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // Delete folder (permission checks are done inside delete_folder)
    delete_folder(&pool, &folder_uuid, &org_uuid, &claims.user_uuid, Some(&dispatcher))
        .await
        .map_err(|e| {
            tracing::error!("Error deleting folder: {}", e);
            match e {
                DocsFolderDatabaseError::UserNotInOrganization => (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "User does not belong to this organization" })),
                ),
                DocsFolderDatabaseError::PermissionDenied => (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "User does not have permission to delete this folder" })),
                ),
                DocsFolderDatabaseError::FolderNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Folder not found" })),
                ),
                DocsFolderDatabaseError::FolderNotInOrganization => (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "Folder does not belong to this organization" })),
                ),
                DocsFolderDatabaseError::FolderNotEmpty => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "Folder cannot be deleted: contains sub-folders or pages" })),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to delete folder" })),
                ),
            }
        })?;

    Ok(Json(json!({
        "message": "Folder deleted successfully"
    })))
}

/// Update folder name
///
/// PUT /api/modules/docs/folders/{uuid}/name
pub async fn update_folder_name_endpoint(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Extension(dispatcher): Extension<EventDispatcher>,
    Path(folder_uuid): Path<String>,
    Json(request): Json<UpdateDocsFolderRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // Get name from request
    let name = request.name.ok_or((
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": "Name is required" })),
    ))?;

    // Sanity checks
    if name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Name cannot be empty" })),
        ));
    }

    if name.len() > 255 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Name cannot exceed 255 characters" })),
        ));
    }

    // Update folder name (permission checks are done inside update_folder_name)
    update_folder_name(&pool, &folder_uuid, &org_uuid, &claims.user_uuid, name, Some(&dispatcher))
        .await
        .map_err(|e| {
            tracing::error!("Error updating folder name: {}", e);
            match e {
                DocsFolderDatabaseError::UserNotInOrganization => (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "User does not belong to this organization" })),
                ),
                DocsFolderDatabaseError::PermissionDenied => (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "User does not have permission to edit this folder" })),
                ),
                DocsFolderDatabaseError::FolderNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Folder not found" })),
                ),
                DocsFolderDatabaseError::FolderNotInOrganization => (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "Folder does not belong to this organization" })),
                ),
                DocsFolderDatabaseError::EmptyName => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "Name cannot be empty" })),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to update folder name" })),
                ),
            }
        })?;

    Ok(Json(json!({
        "message": "Folder name updated successfully"
    })))
}

/// Reorder a folder
///
/// PUT /api/modules/docs/folders/{uuid}/sort-order
pub async fn reorder_folder_endpoint(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Extension(dispatcher): Extension<EventDispatcher>,
    Path(folder_uuid): Path<String>,
    Json(request): Json<UpdateDocsFolderRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // Get sort_order from request
    let sort_order = request.sort_order.ok_or((
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": "sort_order is required" })),
    ))?;

    // Reorder folder (permission checks are done inside reorder_folder)
    reorder_folder(&pool, &folder_uuid, &org_uuid, &claims.user_uuid, sort_order, Some(&dispatcher))
        .await
        .map_err(|e| {
            tracing::error!("Error reordering folder: {}", e);
            match e {
                DocsFolderDatabaseError::UserNotInOrganization => (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "User does not belong to this organization" })),
                ),
                DocsFolderDatabaseError::PermissionDenied => (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "User does not have permission to edit this folder" })),
                ),
                DocsFolderDatabaseError::FolderNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Folder not found" })),
                ),
                DocsFolderDatabaseError::FolderNotInOrganization => (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "Folder does not belong to this organization" })),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to reorder folder" })),
                ),
            }
        })?;

    Ok(Json(json!({
        "message": "Folder reordered successfully"
    })))
}

