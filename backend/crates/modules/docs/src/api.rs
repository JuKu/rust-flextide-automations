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
    create_area, delete_area, load_area_by_uuid, load_area_member_permissions, list_accessible_areas, update_area,
    CreateDocsAreaRequest, DocsAreaDatabaseError, UpdateDocsAreaRequest,
};
use crate::folder::{
    create_folder, delete_folder, list_folders, move_folder, reorder_folder, update_folder, update_folder_name,
    update_folder_properties,
    CreateDocsFolderRequest, DocsFolderDatabaseError, MoveDocsFolderRequest, UpdateDocsFolderRequest,
};
use crate::page::{list_pages, DocsPageDatabaseError};
use crate::tree::{get_area_tree, DocsTreeError};
use flextide_core::user::{user_belongs_to_organization, user_has_permission};

/// Create the API router for Docs endpoints
pub fn create_api_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/modules/docs/documents", get(list_documents))
        .route("/modules/docs/areas", get(list_areas_endpoint).post(create_area_endpoint))
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
        .route("/modules/docs/folders/{uuid}", put(update_folder_endpoint))
        .route("/modules/docs/folders/{uuid}/name", put(update_folder_name_endpoint))
        .route("/modules/docs/folders/{uuid}/properties", put(update_folder_properties_endpoint))
        .route(
            "/modules/docs/folders/{uuid}/sort-order",
            put(reorder_folder_endpoint),
        )
        .route(
            "/modules/docs/folders/{uuid}/move",
            put(move_folder_endpoint),
        )
        .route("/modules/docs/activity", get(list_activity_endpoint))
        .route("/modules/docs/areas/{area_uuid}/tree", get(get_area_tree_endpoint))
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

/// List all accessible areas for the current user
///
/// GET /api/modules/docs/areas
pub async fn list_areas_endpoint(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
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

    // List accessible areas
    let areas = list_accessible_areas(&pool, &org_uuid, &claims.user_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error listing areas: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    Ok(Json(json!({
        "areas": areas
    })))
}

/// List activity feed (mock data for now)
///
/// GET /api/modules/docs/activity
pub async fn list_activity_endpoint(
    Extension(_pool): Extension<DatabasePool>,
    Extension(_org_uuid): Extension<String>,
    Extension(_claims): Extension<Claims>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // TODO: Implement real activity feed from database
    // For now, return mock data
    let activities = vec![
        json!({
            "id": "1",
            "type": "page_created",
            "user_name": "John Doe",
            "user_uuid": "00000000-0000-0000-0000-000000000001",
            "page_title": "Getting Started Guide",
            "page_uuid": "00000000-0000-0000-0000-000000000010",
            "area_name": "Documentation",
            "area_uuid": "00000000-0000-0000-0000-000000000020",
            "timestamp": "2024-01-15T10:30:00Z"
        }),
        json!({
            "id": "2",
            "type": "page_updated",
            "user_name": "Jane Smith",
            "user_uuid": "00000000-0000-0000-0000-000000000002",
            "page_title": "API Reference",
            "page_uuid": "00000000-0000-0000-0000-000000000011",
            "area_name": "Documentation",
            "area_uuid": "00000000-0000-0000-0000-000000000020",
            "timestamp": "2024-01-15T09:15:00Z"
        }),
        json!({
            "id": "3",
            "type": "page_created",
            "user_name": "Bob Johnson",
            "user_uuid": "00000000-0000-0000-0000-000000000003",
            "page_title": "Installation Guide",
            "page_uuid": "00000000-0000-0000-0000-000000000012",
            "area_name": "User Guides",
            "area_uuid": "00000000-0000-0000-0000-000000000021",
            "timestamp": "2024-01-14T14:20:00Z"
        }),
    ];

    Ok(Json(json!({
        "activities": activities
    })))
}

/// Query parameters for listing pages
#[derive(Debug, Deserialize)]
pub(crate) struct ListPagesQuery {
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
pub(crate) struct ListFoldersQuery {
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

/// Update a folder
///
/// PUT /api/modules/docs/folders/{uuid}
pub async fn update_folder_endpoint(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Extension(dispatcher): Extension<EventDispatcher>,
    Path(folder_uuid): Path<String>,
    Json(mut request): Json<UpdateDocsFolderRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    tracing::info!(
        "Updating folder: folder_uuid={}, organization_uuid={}, user_uuid={}",
        folder_uuid,
        org_uuid,
        claims.user_uuid
    );

    // Validate name if provided
    if let Some(ref name) = request.name {
        if name.trim().is_empty() {
            tracing::warn!(
                "Folder update failed: empty name provided for folder_uuid={}",
                folder_uuid
            );
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Name cannot be empty" })),
            ));
        }

        if name.len() > 255 {
            tracing::warn!(
                "Folder update failed: name too long ({} chars) for folder_uuid={}",
                name.len(),
                folder_uuid
            );
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Name cannot exceed 255 characters" })),
            ));
        }
    }

    // Sanitize icon_name: trim whitespace, use "folder" as default if empty
    request.icon_name = request.icon_name.as_ref()
        .map(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                "folder".to_string()
            } else {
                trimmed.to_string()
            }
        })
        .or(Some("folder".to_string()));

    // Sanitize folder_color: ensure it starts with # if provided, validate hex format, default to #000000 if invalid
    request.folder_color = request.folder_color.as_ref()
        .and_then(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                Some("#000000".to_string())
            } else {
                let color = if trimmed.starts_with('#') {
                    trimmed.to_string()
                } else {
                    format!("#{}", trimmed)
                };
                
                // Validate hex color format: must have 6 or 8 hex digits after #
                let hex_part = &color[1..];
                if hex_part.len() == 6 || hex_part.len() == 8 {
                    // Check if all characters are valid hex (0-9, a-f, A-F)
                    if hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
                        Some(color)
                    } else {
                        Some("#000000".to_string()) // Invalid hex characters, use black
                    }
                } else {
                    Some("#000000".to_string()) // Invalid length, use black
                }
            }
        })
        .or(Some("#000000".to_string())); // If folder_color was None, use black

    // Log sanitized values
    tracing::debug!(
        "Folder update request sanitized: folder_uuid={}, name={:?}, icon_name={:?}, folder_color={:?}",
        folder_uuid,
        request.name,
        request.icon_name,
        request.folder_color
    );

    // Update folder (permission checks are done inside update_folder)
    update_folder(&pool, &folder_uuid, &org_uuid, &claims.user_uuid, request, Some(&dispatcher))
        .await
        .map_err(|e| {
            tracing::error!(
                "Error updating folder: folder_uuid={}, organization_uuid={}, user_uuid={}, error={}",
                folder_uuid,
                org_uuid,
                claims.user_uuid,
                e
            );
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
                    Json(json!({ "error": "Failed to update folder" })),
                ),
            }
        })?;

    tracing::info!(
        "Folder updated successfully: folder_uuid={}, organization_uuid={}, user_uuid={}",
        folder_uuid,
        org_uuid,
        claims.user_uuid
    );

    Ok(Json(json!({
        "message": "Folder updated successfully"
    })))
}

/// Update folder properties
///
/// PUT /api/modules/docs/folders/{uuid}/properties
pub async fn update_folder_properties_endpoint(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Extension(dispatcher): Extension<EventDispatcher>,
    Path(folder_uuid): Path<String>,
    Json(request): Json<serde_json::Value>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // Extract fields from request
    let auto_sync_to_vector_db = request
        .get("auto_sync_to_vector_db")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    let vcs_export_allowed = request
        .get("vcs_export_allowed")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    let includes_private_data = request
        .get("includes_private_data")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    let metadata = request
        .get("metadata")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    // Update folder properties (permission checks are done inside update_folder_properties)
    update_folder_properties(
        &pool,
        &folder_uuid,
        &org_uuid,
        &claims.user_uuid,
        auto_sync_to_vector_db,
        vcs_export_allowed,
        includes_private_data,
        metadata,
        Some(&dispatcher),
    )
    .await
    .map_err(|e| {
        tracing::error!("Error updating folder properties: {}", e);
        match e {
            DocsFolderDatabaseError::UserNotInOrganization => (
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "User does not belong to this organization" })),
            ),
            DocsFolderDatabaseError::PermissionDenied => (
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "User does not have permission to edit folder properties" })),
            ),
            DocsFolderDatabaseError::FolderNotFound => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Folder not found" })),
            ),
            DocsFolderDatabaseError::FolderNotInOrganization => (
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "Folder does not belong to this organization" })),
            ),
            DocsFolderDatabaseError::InvalidMetadata => (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Metadata must be a valid JSON object" })),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to update folder properties" })),
            ),
        }
    })?;

    Ok(Json(json!({
        "message": "Folder properties updated successfully"
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

/// Move a folder to a different parent and/or position
///
/// PUT /api/modules/docs/folders/{uuid}/move
pub async fn move_folder_endpoint(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Extension(dispatcher): Extension<EventDispatcher>,
    Path(folder_uuid): Path<String>,
    Json(request): Json<MoveDocsFolderRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // Move folder (permission checks are done inside move_folder)
    move_folder(
        &pool,
        &folder_uuid,
        &org_uuid,
        &claims.user_uuid,
        request.parent_folder_uuid,
        request.sort_order,
        Some(&dispatcher),
    )
    .await
    .map_err(|e| {
        tracing::error!("Error moving folder: {}", e);
        match e {
            DocsFolderDatabaseError::UserNotInOrganization => (
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "User does not belong to this organization" })),
            ),
            DocsFolderDatabaseError::PermissionDenied => (
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "User does not have permission to move this folder" })),
            ),
            DocsFolderDatabaseError::FolderNotFound => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Folder not found" })),
            ),
            DocsFolderDatabaseError::FolderNotInOrganization => (
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "Folder does not belong to this organization" })),
            ),
            DocsFolderDatabaseError::AreaNotInOrganization => (
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "Area does not belong to this organization" })),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to move folder" })),
            ),
        }
    })?;

    Ok(Json(json!({
        "message": "Folder moved successfully"
    })))
}

/// Get the folder tree structure for an area (with nested folders and pages)
///
/// GET /api/modules/docs/areas/{area_uuid}/tree
pub async fn get_area_tree_endpoint(
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

    // Check if user has super_admin permission (grants access to everything)
    let has_super_admin = user_has_permission(&pool, &claims.user_uuid, &org_uuid, "super_admin")
        .await
        .map_err(|e| {
            tracing::error!("Database error checking super_admin permission: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    // Check if user is a member of the area OR has super_admin permission
    let member_perms = load_area_member_permissions(&pool, &area_uuid, &claims.user_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error loading area member permissions: {}", e);
            match e {
                DocsAreaDatabaseError::Database(_) | DocsAreaDatabaseError::Sql(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Database error" })),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to check area membership" })),
                ),
            }
        })?;

    let is_member = member_perms.is_some();
    
    if !is_member && !has_super_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "User is not a member of this area" })),
        ));
    }

    // Check if user has can_view permission in the area OR has super_admin permission
    let can_view = if has_super_admin {
        true
    } else if let Some(perms) = &member_perms {
        perms.can_view || perms.admin || perms.role == "owner"
    } else {
        false
    };

    if !can_view {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "User does not have permission to view this area" })),
        ));
    }

    // Get the tree structure
    let tree = get_area_tree(&pool, &org_uuid, &area_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Error building area tree: {}", e);
            match e {
                DocsTreeError::FolderError(folder_err) => match folder_err {
                    DocsFolderDatabaseError::Database(_) | DocsFolderDatabaseError::Sql(_) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Database error" })),
                    ),
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Failed to load folders" })),
                    ),
                },
                DocsTreeError::PageError(page_err) => match page_err {
                    DocsPageDatabaseError::Database(_) | DocsPageDatabaseError::Sql(_) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Database error" })),
                    ),
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Failed to load pages" })),
                    ),
                },
            }
        })?;

    Ok(Json(json!({
        "tree": tree
    })))
}

