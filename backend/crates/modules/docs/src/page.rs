//! Docs Page module
//!
//! Provides functionality for managing documentation pages, including database operations and permission checks.

use chrono::{DateTime, Utc};
use flextide_core::database::{DatabaseError, DatabasePool};
use flextide_core::events::{Event, EventDispatcher, EventPayload};
use flextide_core::user::{user_belongs_to_organization, user_has_permission};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use sqlx::Row;
use thiserror::Error;

use crate::area::{
    load_area_by_uuid, load_area_member_permissions, AreaMemberPermissions, DocsAreaDatabaseError,
};

/// Error type for Docs page database operations
#[derive(Debug, Error)]
pub enum DocsPageDatabaseError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("SQL execution error: {0}")]
    Sql(#[from] sqlx::Error),

    #[error("User does not belong to this organization")]
    UserNotInOrganization,

    #[error("User does not have permission to perform this action")]
    PermissionDenied,

    #[error("Page not found")]
    PageNotFound,

    #[error("Page does not belong to this organization")]
    PageNotInOrganization,

    #[error("Area not found")]
    AreaNotFound,

    #[error("Area does not belong to this organization")]
    AreaNotInOrganization,

    #[error("Title cannot be empty")]
    EmptyTitle,
}

/// Docs Page data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsPage {
    pub uuid: String,
    pub organization_uuid: String,
    pub area_uuid: String,
    pub folder_uuid: Option<String>,
    pub title: String,
    pub short_summary: Option<String>,
    pub parent_page_uuid: Option<String>,
    pub current_version_uuid: Option<String>,
    pub page_type: String,
    pub last_updated: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub auto_sync_to_vector_db: i32,
    pub vcs_export_allowed: i32,
    pub includes_private_data: i32,
    pub metadata: Option<JsonValue>,
}

/// Request structure for creating a new page
#[derive(Debug, Deserialize)]
pub struct CreateDocsPageRequest {
    pub area_uuid: String,
    pub title: String,
    pub short_summary: Option<String>,
    pub parent_page_uuid: Option<String>,
    pub page_type: Option<String>,
}

/// Docs Page Version data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsPageVersion {
    pub uuid: String,
    pub page_uuid: String,
    pub version_number: i32,
    pub content: String,
    pub last_updated: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Docs Page with its current version combined
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsPageWithVersion {
    pub uuid: String,
    pub organization_uuid: String,
    pub area_uuid: String,
    pub folder_uuid: Option<String>,
    pub title: String,
    pub short_summary: Option<String>,
    pub parent_page_uuid: Option<String>,
    pub current_version_uuid: Option<String>,
    pub page_type: String,
    pub last_updated: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub auto_sync_to_vector_db: i32,
    pub vcs_export_allowed: i32,
    pub includes_private_data: i32,
    pub metadata: Option<JsonValue>,
    pub version: Option<DocsPageVersion>,
}


/// Load a page by UUID
async fn load_page_by_uuid(
    pool: &DatabasePool,
    page_uuid: &str,
) -> Result<DocsPage, DocsPageDatabaseError> {
    match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query(
                "SELECT uuid, organization_uuid, area_uuid, folder_uuid, title, short_summary, parent_page_uuid,
                 current_version_uuid, page_type, last_updated, created_at, auto_sync_to_vector_db,
                 vcs_export_allowed, includes_private_data, metadata
                 FROM module_docs_pages WHERE uuid = ?",
            )
            .bind(page_uuid)
            .fetch_optional(p)
            .await?;

            match row {
                Some(row) => Ok(DocsPage {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    area_uuid: row.get("area_uuid"),
                    folder_uuid: row.get("folder_uuid"),
                    title: row.get("title"),
                    short_summary: row.get("short_summary"),
                    parent_page_uuid: row.get("parent_page_uuid"),
                    current_version_uuid: row.get("current_version_uuid"),
                    page_type: row.get("page_type"),
                    last_updated: row.get::<DateTime<Utc>, _>("last_updated"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    auto_sync_to_vector_db: row.get("auto_sync_to_vector_db"),
                    vcs_export_allowed: row.get("vcs_export_allowed"),
                    includes_private_data: row.get("includes_private_data"),
                    metadata: row.get("metadata"),
                }),
                None => Err(DocsPageDatabaseError::PageNotFound),
            }
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT uuid, organization_uuid, area_uuid, folder_uuid, title, short_summary, parent_page_uuid,
                 current_version_uuid, page_type, last_updated, created_at, auto_sync_to_vector_db,
                 vcs_export_allowed, includes_private_data, metadata
                 FROM module_docs_pages WHERE uuid = $1",
            )
            .bind(page_uuid)
            .fetch_optional(p)
            .await?;

            match row {
                Some(row) => Ok(DocsPage {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    area_uuid: row.get("area_uuid"),
                    folder_uuid: row.get("folder_uuid"),
                    title: row.get("title"),
                    short_summary: row.get("short_summary"),
                    parent_page_uuid: row.get("parent_page_uuid"),
                    current_version_uuid: row.get("current_version_uuid"),
                    page_type: row.get("page_type"),
                    last_updated: row.get::<DateTime<Utc>, _>("last_updated"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    auto_sync_to_vector_db: row.get("auto_sync_to_vector_db"),
                    vcs_export_allowed: row.get("vcs_export_allowed"),
                    includes_private_data: row.get("includes_private_data"),
                    metadata: row.get("metadata"),
                }),
                None => Err(DocsPageDatabaseError::PageNotFound),
            }
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT uuid, organization_uuid, area_uuid, folder_uuid, title, short_summary, parent_page_uuid,
                 current_version_uuid, page_type, last_updated, created_at, auto_sync_to_vector_db,
                 vcs_export_allowed, includes_private_data, metadata
                 FROM module_docs_pages WHERE uuid = ?1",
            )
            .bind(page_uuid)
            .fetch_optional(p)
            .await?;

            match row {
                Some(row) => Ok(DocsPage {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    area_uuid: row.get("area_uuid"),
                    folder_uuid: row.get("folder_uuid"),
                    title: row.get("title"),
                    short_summary: row.get("short_summary"),
                    parent_page_uuid: row.get("parent_page_uuid"),
                    current_version_uuid: row.get("current_version_uuid"),
                    page_type: row.get("page_type"),
                    last_updated: row.get::<DateTime<Utc>, _>("last_updated"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    auto_sync_to_vector_db: row.get("auto_sync_to_vector_db"),
                    vcs_export_allowed: row.get("vcs_export_allowed"),
                    includes_private_data: row.get("includes_private_data"),
                    metadata: row.get("metadata"),
                }),
                None => Err(DocsPageDatabaseError::PageNotFound),
            }
        }
    }
}

/// Create a new page in the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `organization_uuid` - UUID of the organization the page belongs to
/// * `user_uuid` - UUID of the user creating the page
/// * `request` - Page creation request
///
/// # Returns
/// Returns the UUID of the newly created page
///
/// # Errors
/// Returns `DocsPageDatabaseError` if:
/// - User does not belong to the organization
/// - User does not have permission to create pages
/// - Area does not belong to the organization
/// - Title is empty
/// - Database operation fails
pub async fn create_page(
    pool: &DatabasePool,
    organization_uuid: &str,
    user_uuid: &str,
    request: CreateDocsPageRequest,
    dispatcher: Option<&EventDispatcher>,
) -> Result<String, DocsPageDatabaseError> {
    // Validate title
    if request.title.trim().is_empty() {
        return Err(DocsPageDatabaseError::EmptyTitle);
    }

    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(pool, user_uuid, organization_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            DocsPageDatabaseError::Database(e.into())
        })?;

    if !belongs {
        return Err(DocsPageDatabaseError::UserNotInOrganization);
    }

    // Load area to verify it belongs to the organization
    let area = load_area_by_uuid(pool, &request.area_uuid)
        .await
        .map_err(|e| match e {
            DocsAreaDatabaseError::AreaNotFound => DocsPageDatabaseError::AreaNotFound,
            DocsAreaDatabaseError::Database(e) => DocsPageDatabaseError::Database(e),
            DocsAreaDatabaseError::Sql(e) => DocsPageDatabaseError::Sql(e),
            DocsAreaDatabaseError::UserNotInOrganization => {
                DocsPageDatabaseError::UserNotInOrganization
            }
            DocsAreaDatabaseError::PermissionDenied => DocsPageDatabaseError::PermissionDenied,
            DocsAreaDatabaseError::AreaNotInOrganization => {
                DocsPageDatabaseError::AreaNotInOrganization
            }
            DocsAreaDatabaseError::EmptyShortName => {
                DocsPageDatabaseError::Database(
                    flextide_core::database::DatabaseError::PoolCreationFailed(
                        sqlx::Error::RowNotFound,
                    ),
                )
            }
        })?;

    if area.organization_uuid != organization_uuid {
        return Err(DocsPageDatabaseError::AreaNotInOrganization);
    }

    // Check area member permissions
    let member_perms = load_area_member_permissions(pool, &request.area_uuid, user_uuid)
        .await
        .map_err(|e| match e {
            DocsAreaDatabaseError::Database(e) => DocsPageDatabaseError::Database(e),
            DocsAreaDatabaseError::Sql(e) => DocsPageDatabaseError::Sql(e),
            DocsAreaDatabaseError::UserNotInOrganization => {
                DocsPageDatabaseError::UserNotInOrganization
            }
            DocsAreaDatabaseError::PermissionDenied => DocsPageDatabaseError::PermissionDenied,
            DocsAreaDatabaseError::AreaNotFound => DocsPageDatabaseError::AreaNotFound,
            DocsAreaDatabaseError::AreaNotInOrganization => {
                DocsPageDatabaseError::AreaNotInOrganization
            }
            DocsAreaDatabaseError::EmptyShortName => {
                DocsPageDatabaseError::Database(
                    flextide_core::database::DatabaseError::PoolCreationFailed(
                        sqlx::Error::RowNotFound,
                    ),
                )
            }
        })?;

    // Check if user has permission to add pages
    // Allow if: user is admin/owner in area, or has can_add_pages permission
    let can_add = if let Some(perms) = &member_perms {
        perms.admin || perms.role == "owner" || perms.can_add_pages
    } else {
        // If not a member, check if area is public and user has organization-level permission
        if area.public {
            user_has_permission(
                pool,
                user_uuid,
                organization_uuid,
                "module_docs_can_create_areas", // Using area creation permission as fallback
            )
            .await
            .map_err(|e| {
                tracing::error!("Database error checking permission: {}", e);
                match e {
                    flextide_core::user::UserDatabaseError::Database(db_err) => {
                        DocsPageDatabaseError::Database(db_err)
                    }
                    flextide_core::user::UserDatabaseError::Sql(sql_err) => {
                        DocsPageDatabaseError::Sql(sql_err)
                    }
                    _ => DocsPageDatabaseError::Database(
                        flextide_core::database::DatabaseError::PoolCreationFailed(
                            sqlx::Error::RowNotFound,
                        ),
                    ),
                }
            })?
        } else {
            false
        }
    };

    if !can_add {
        return Err(DocsPageDatabaseError::PermissionDenied);
    }

    // Create page
    let page_uuid = uuid::Uuid::new_v4().to_string();
    let page_type = request
        .page_type
        .unwrap_or_else(|| "markdown_page".to_string());

    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query(
                "INSERT INTO module_docs_pages (uuid, organization_uuid, area_uuid, title, short_summary, parent_page_uuid, page_type)
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&page_uuid)
            .bind(organization_uuid)
            .bind(&request.area_uuid)
            .bind(&request.title)
            .bind(&request.short_summary)
            .bind(&request.parent_page_uuid)
            .bind(&page_type)
            .execute(p)
            .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO module_docs_pages (uuid, organization_uuid, area_uuid, title, short_summary, parent_page_uuid, page_type)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(&page_uuid)
            .bind(organization_uuid)
            .bind(&request.area_uuid)
            .bind(&request.title)
            .bind(&request.short_summary)
            .bind(&request.parent_page_uuid)
            .bind(&page_type)
            .execute(p)
            .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO module_docs_pages (uuid, organization_uuid, area_uuid, title, short_summary, parent_page_uuid, page_type)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            )
            .bind(&page_uuid)
            .bind(organization_uuid)
            .bind(&request.area_uuid)
            .bind(&request.title)
            .bind(&request.short_summary)
            .bind(&request.parent_page_uuid)
            .bind(&page_type)
            .execute(p)
            .await?;
        }
    }

    // Emit page created event
    if let Some(disp) = dispatcher {
        let page = load_page_by_uuid(pool, &page_uuid).await.ok();
        let event = Event::new(
            "module_docs_page_created",
            EventPayload::new(json!({
                "entity_type": "page",
                "entity_id": page_uuid,
                "organization_uuid": organization_uuid,
                "data": page.as_ref().map(|p| json!({
                    "title": p.title,
                    "short_summary": p.short_summary,
                    "area_uuid": p.area_uuid,
                    "folder_uuid": p.folder_uuid,
                    "parent_page_uuid": p.parent_page_uuid,
                    "page_type": p.page_type
                })).unwrap_or(json!({}))
            }))
        )
        .with_organization(organization_uuid)
        .with_user(user_uuid);

        disp.emit(event).await;
    }

    Ok(page_uuid)
}

/// Delete a page from the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `page_uuid` - UUID of the page to delete
/// * `organization_uuid` - UUID of the organization
/// * `user_uuid` - UUID of the user deleting the page
///
/// # Returns
/// Returns `()` on success
///
/// # Errors
/// Returns `DocsPageDatabaseError` if:
/// - User does not belong to the organization
/// - User does not have permission to delete pages
/// - Page does not belong to the organization
/// - Page not found
/// - Database operation fails
pub async fn delete_page(
    pool: &DatabasePool,
    page_uuid: &str,
    organization_uuid: &str,
    user_uuid: &str,
    dispatcher: Option<&EventDispatcher>,
) -> Result<(), DocsPageDatabaseError> {
    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(pool, user_uuid, organization_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            DocsPageDatabaseError::Database(e.into())
        })?;

    if !belongs {
        return Err(DocsPageDatabaseError::UserNotInOrganization);
    }

    // Load page to verify it belongs to the organization
    // Also load it before deletion for event payload
    let page = load_page_by_uuid(pool, page_uuid).await?;

    if page.organization_uuid != organization_uuid {
        return Err(DocsPageDatabaseError::PageNotInOrganization);
    }

    // Load area to check permissions
    let area = load_area_by_uuid(pool, &page.area_uuid)
        .await
        .map_err(|e| match e {
            DocsAreaDatabaseError::AreaNotFound => DocsPageDatabaseError::AreaNotFound,
            DocsAreaDatabaseError::Database(e) => DocsPageDatabaseError::Database(e),
            DocsAreaDatabaseError::Sql(e) => DocsPageDatabaseError::Sql(e),
            DocsAreaDatabaseError::UserNotInOrganization => {
                DocsPageDatabaseError::UserNotInOrganization
            }
            DocsAreaDatabaseError::PermissionDenied => DocsPageDatabaseError::PermissionDenied,
            DocsAreaDatabaseError::AreaNotInOrganization => {
                DocsPageDatabaseError::AreaNotInOrganization
            }
            DocsAreaDatabaseError::EmptyShortName => {
                DocsPageDatabaseError::Database(
                    flextide_core::database::DatabaseError::PoolCreationFailed(
                        sqlx::Error::RowNotFound,
                    ),
                )
            }
        })?;

    if area.organization_uuid != organization_uuid {
        return Err(DocsPageDatabaseError::AreaNotInOrganization);
    }

    // Check area member permissions
    let member_perms = load_area_member_permissions(pool, &page.area_uuid, user_uuid)
        .await
        .map_err(|e| match e {
            DocsAreaDatabaseError::Database(e) => DocsPageDatabaseError::Database(e),
            DocsAreaDatabaseError::Sql(e) => DocsPageDatabaseError::Sql(e),
            DocsAreaDatabaseError::UserNotInOrganization => {
                DocsPageDatabaseError::UserNotInOrganization
            }
            DocsAreaDatabaseError::PermissionDenied => DocsPageDatabaseError::PermissionDenied,
            DocsAreaDatabaseError::AreaNotFound => DocsPageDatabaseError::AreaNotFound,
            DocsAreaDatabaseError::AreaNotInOrganization => {
                DocsPageDatabaseError::AreaNotInOrganization
            }
            DocsAreaDatabaseError::EmptyShortName => {
                DocsPageDatabaseError::Database(
                    flextide_core::database::DatabaseError::PoolCreationFailed(
                        sqlx::Error::RowNotFound,
                    ),
                )
            }
        })?;

    // Check if user has permission to delete pages
    // Allow if: user is admin/owner in area, or has can_delete_pages permission,
    // or (has can_delete_own_pages and user is creator - but we don't track creator yet)
    let can_delete = if let Some(perms) = &member_perms {
        perms.admin || perms.role == "owner" || perms.can_delete_pages
        // TODO: Add check for can_delete_own_pages when creator_uuid is added to pages
    } else {
        // If not a member, check if area is public and user has organization-level permission
        if area.public {
            user_has_permission(
                pool,
                user_uuid,
                organization_uuid,
                "module_docs_can_delete_areas", // Using area deletion permission as fallback
            )
            .await
            .map_err(|e| {
                tracing::error!("Database error checking permission: {}", e);
                match e {
                    flextide_core::user::UserDatabaseError::Database(db_err) => {
                        DocsPageDatabaseError::Database(db_err)
                    }
                    flextide_core::user::UserDatabaseError::Sql(sql_err) => {
                        DocsPageDatabaseError::Sql(sql_err)
                    }
                    _ => DocsPageDatabaseError::Database(
                        flextide_core::database::DatabaseError::PoolCreationFailed(
                            sqlx::Error::RowNotFound,
                        ),
                    ),
                }
            })?
        } else {
            false
        }
    };

    if !can_delete {
        return Err(DocsPageDatabaseError::PermissionDenied);
    }

    // Delete page (cascade will delete page_versions)
    match pool {
        DatabasePool::MySql(p) => {
            let result = sqlx::query("DELETE FROM module_docs_pages WHERE uuid = ?")
                .bind(page_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsPageDatabaseError::PageNotFound);
            }
        }
        DatabasePool::Postgres(p) => {
            let result = sqlx::query("DELETE FROM module_docs_pages WHERE uuid = $1")
                .bind(page_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsPageDatabaseError::PageNotFound);
            }
        }
        DatabasePool::Sqlite(p) => {
            let result = sqlx::query("DELETE FROM module_docs_pages WHERE uuid = ?1")
                .bind(page_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsPageDatabaseError::PageNotFound);
            }
        }
    }

    // Emit page deleted event (before deletion, we already have the page data)
    if let Some(disp) = dispatcher {
        let event = Event::new(
            "module_docs_page_deleted",
            EventPayload::new(json!({
                "entity_type": "page",
                "entity_id": page_uuid,
                "organization_uuid": organization_uuid,
                "data": json!({
                    "title": page.title,
                    "short_summary": page.short_summary,
                    "area_uuid": page.area_uuid,
                    "folder_uuid": page.folder_uuid,
                    "parent_page_uuid": page.parent_page_uuid,
                    "page_type": page.page_type
                })
            }))
        )
        .with_organization(organization_uuid)
        .with_user(user_uuid);

        disp.emit(event).await;
    }

    Ok(())
}

/// Get permissions for a specific user for a specific page
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `page_uuid` - UUID of the page
/// * `user_uuid` - UUID of the user
///
/// # Returns
/// Returns `Ok(Some(AreaMemberPermissions))` if the user has permissions in the page's area,
/// `Ok(None)` if the user is not a member of the area, or an error if the page doesn't exist
/// or database operation fails.
///
/// # Errors
/// Returns `DocsPageDatabaseError` if:
/// - Page not found
/// - Area not found
/// - Database operation fails
pub async fn get_page_user_permissions(
    pool: &DatabasePool,
    page_uuid: &str,
    user_uuid: &str,
) -> Result<Option<AreaMemberPermissions>, DocsPageDatabaseError> {
    // Load page to get its area_uuid
    let page = load_page_by_uuid(pool, page_uuid).await?;

    // Load area member permissions for the user in the page's area
    let permissions = load_area_member_permissions(pool, &page.area_uuid, user_uuid)
        .await
        .map_err(|e| match e {
            DocsAreaDatabaseError::Database(e) => DocsPageDatabaseError::Database(e),
            DocsAreaDatabaseError::Sql(e) => DocsPageDatabaseError::Sql(e),
            DocsAreaDatabaseError::UserNotInOrganization => {
                DocsPageDatabaseError::UserNotInOrganization
            }
            DocsAreaDatabaseError::PermissionDenied => DocsPageDatabaseError::PermissionDenied,
            DocsAreaDatabaseError::AreaNotFound => DocsPageDatabaseError::AreaNotFound,
            DocsAreaDatabaseError::AreaNotInOrganization => {
                DocsPageDatabaseError::AreaNotInOrganization
            }
            DocsAreaDatabaseError::EmptyShortName => {
                DocsPageDatabaseError::Database(
                    flextide_core::database::DatabaseError::PoolCreationFailed(
                        sqlx::Error::RowNotFound,
                    ),
                )
            }
        })?;

    Ok(permissions)
}

/// List pages for a given organization, area, and optional folder
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `organization_uuid` - UUID of the organization
/// * `area_uuid` - UUID of the area
/// * `folder_uuid` - Optional folder UUID (None for root folder pages)
/// * `user_uuid` - UUID of the user requesting the pages
///
/// # Returns
/// Returns a vector of pages that the user has permission to view
///
/// # Errors
/// Returns `DocsPageDatabaseError` if:
/// - User does not belong to the organization
/// - Area does not belong to the organization
/// - User does not have permission to view pages in the area
/// - Database operation fails
pub async fn list_pages(
    pool: &DatabasePool,
    organization_uuid: &str,
    area_uuid: &str,
    folder_uuid: Option<&str>,
    user_uuid: &str,
) -> Result<Vec<DocsPage>, DocsPageDatabaseError> {
    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(pool, user_uuid, organization_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            DocsPageDatabaseError::Database(e.into())
        })?;

    if !belongs {
        return Err(DocsPageDatabaseError::UserNotInOrganization);
    }

    // Load area to verify it belongs to the organization
    let area = load_area_by_uuid(pool, area_uuid)
        .await
        .map_err(|e| match e {
            DocsAreaDatabaseError::AreaNotFound => DocsPageDatabaseError::AreaNotFound,
            DocsAreaDatabaseError::Database(e) => DocsPageDatabaseError::Database(e),
            DocsAreaDatabaseError::Sql(e) => DocsPageDatabaseError::Sql(e),
            DocsAreaDatabaseError::UserNotInOrganization => {
                DocsPageDatabaseError::UserNotInOrganization
            }
            DocsAreaDatabaseError::PermissionDenied => DocsPageDatabaseError::PermissionDenied,
            DocsAreaDatabaseError::AreaNotInOrganization => {
                DocsPageDatabaseError::AreaNotInOrganization
            }
            DocsAreaDatabaseError::EmptyShortName => {
                DocsPageDatabaseError::Database(
                    flextide_core::database::DatabaseError::PoolCreationFailed(
                        sqlx::Error::RowNotFound,
                    ),
                )
            }
        })?;

    if area.organization_uuid != organization_uuid {
        return Err(DocsPageDatabaseError::AreaNotInOrganization);
    }

    // Check area member permissions
    let member_perms = load_area_member_permissions(pool, area_uuid, user_uuid)
        .await
        .map_err(|e| match e {
            DocsAreaDatabaseError::Database(e) => DocsPageDatabaseError::Database(e),
            DocsAreaDatabaseError::Sql(e) => DocsPageDatabaseError::Sql(e),
            DocsAreaDatabaseError::UserNotInOrganization => {
                DocsPageDatabaseError::UserNotInOrganization
            }
            DocsAreaDatabaseError::PermissionDenied => DocsPageDatabaseError::PermissionDenied,
            DocsAreaDatabaseError::AreaNotFound => DocsPageDatabaseError::AreaNotFound,
            DocsAreaDatabaseError::AreaNotInOrganization => {
                DocsPageDatabaseError::AreaNotInOrganization
            }
            DocsAreaDatabaseError::EmptyShortName => {
                DocsPageDatabaseError::Database(
                    flextide_core::database::DatabaseError::PoolCreationFailed(
                        sqlx::Error::RowNotFound,
                    ),
                )
            }
        })?;

    // Check if user has permission to view pages
    let can_view = if let Some(perms) = &member_perms {
        perms.admin || perms.role == "owner" || perms.can_view
    } else {
        // If not a member, check if area is public and user has organization-level permission
        if area.public {
            user_has_permission(
                pool,
                user_uuid,
                organization_uuid,
                "module_docs_can_create_areas", // Using area creation permission as fallback
            )
            .await
            .map_err(|e| {
                tracing::error!("Database error checking permission: {}", e);
                match e {
                    flextide_core::user::UserDatabaseError::Database(db_err) => {
                        DocsPageDatabaseError::Database(db_err)
                    }
                    flextide_core::user::UserDatabaseError::Sql(sql_err) => {
                        DocsPageDatabaseError::Sql(sql_err)
                    }
                    _ => DocsPageDatabaseError::Database(
                        flextide_core::database::DatabaseError::PoolCreationFailed(
                            sqlx::Error::RowNotFound,
                        ),
                    ),
                }
            })?
        } else {
            false
        }
    };

    if !can_view {
        return Err(DocsPageDatabaseError::PermissionDenied);
    }

    // Query pages with folder filter
    match pool {
        DatabasePool::MySql(p) => {
            let pages = if let Some(folder) = folder_uuid {
                sqlx::query(
                    "SELECT uuid, organization_uuid, area_uuid, folder_uuid, title, short_summary, parent_page_uuid,
                     current_version_uuid, page_type, last_updated, created_at, auto_sync_to_vector_db,
                     vcs_export_allowed, includes_private_data, metadata
                     FROM module_docs_pages
                     WHERE organization_uuid = ? AND area_uuid = ? AND folder_uuid = ?
                     ORDER BY created_at DESC",
                )
                .bind(organization_uuid)
                .bind(area_uuid)
                .bind(folder)
                .fetch_all(p)
                .await?
            } else {
                sqlx::query(
                    "SELECT uuid, organization_uuid, area_uuid, folder_uuid, title, short_summary, parent_page_uuid,
                     current_version_uuid, page_type, last_updated, created_at, auto_sync_to_vector_db,
                     vcs_export_allowed, includes_private_data, metadata
                     FROM module_docs_pages
                     WHERE organization_uuid = ? AND area_uuid = ? AND folder_uuid IS NULL
                     ORDER BY created_at DESC",
                )
                .bind(organization_uuid)
                .bind(area_uuid)
                .fetch_all(p)
                .await?
            };

            Ok(pages
                .into_iter()
                .map(|row| DocsPage {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    area_uuid: row.get("area_uuid"),
                    folder_uuid: row.get("folder_uuid"),
                    title: row.get("title"),
                    short_summary: row.get("short_summary"),
                    parent_page_uuid: row.get("parent_page_uuid"),
                    current_version_uuid: row.get("current_version_uuid"),
                    page_type: row.get("page_type"),
                    last_updated: row.get::<DateTime<Utc>, _>("last_updated"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    auto_sync_to_vector_db: row.get("auto_sync_to_vector_db"),
                    vcs_export_allowed: row.get("vcs_export_allowed"),
                    includes_private_data: row.get("includes_private_data"),
                    metadata: row.get("metadata"),
                })
                .collect())
        }
        DatabasePool::Postgres(p) => {
            let pages = if let Some(folder) = folder_uuid {
                sqlx::query(
                    "SELECT uuid, organization_uuid, area_uuid, folder_uuid, title, short_summary, parent_page_uuid,
                     current_version_uuid, page_type, last_updated, created_at, auto_sync_to_vector_db,
                     vcs_export_allowed, includes_private_data, metadata
                     FROM module_docs_pages
                     WHERE organization_uuid = $1 AND area_uuid = $2 AND folder_uuid = $3
                     ORDER BY created_at DESC",
                )
                .bind(organization_uuid)
                .bind(area_uuid)
                .bind(folder)
                .fetch_all(p)
                .await?
            } else {
                sqlx::query(
                    "SELECT uuid, organization_uuid, area_uuid, folder_uuid, title, short_summary, parent_page_uuid,
                     current_version_uuid, page_type, last_updated, created_at, auto_sync_to_vector_db,
                     vcs_export_allowed, includes_private_data, metadata
                     FROM module_docs_pages
                     WHERE organization_uuid = $1 AND area_uuid = $2 AND folder_uuid IS NULL
                     ORDER BY created_at DESC",
                )
                .bind(organization_uuid)
                .bind(area_uuid)
                .fetch_all(p)
                .await?
            };

            Ok(pages
                .into_iter()
                .map(|row| DocsPage {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    area_uuid: row.get("area_uuid"),
                    folder_uuid: row.get("folder_uuid"),
                    title: row.get("title"),
                    short_summary: row.get("short_summary"),
                    parent_page_uuid: row.get("parent_page_uuid"),
                    current_version_uuid: row.get("current_version_uuid"),
                    page_type: row.get("page_type"),
                    last_updated: row.get::<DateTime<Utc>, _>("last_updated"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    auto_sync_to_vector_db: row.get("auto_sync_to_vector_db"),
                    vcs_export_allowed: row.get("vcs_export_allowed"),
                    includes_private_data: row.get("includes_private_data"),
                    metadata: row.get("metadata"),
                })
                .collect())
        }
        DatabasePool::Sqlite(p) => {
            let pages = if let Some(folder) = folder_uuid {
                sqlx::query(
                    "SELECT uuid, organization_uuid, area_uuid, folder_uuid, title, short_summary, parent_page_uuid,
                     current_version_uuid, page_type, last_updated, created_at, auto_sync_to_vector_db,
                     vcs_export_allowed, includes_private_data, metadata
                     FROM module_docs_pages
                     WHERE organization_uuid = ?1 AND area_uuid = ?2 AND folder_uuid = ?3
                     ORDER BY created_at DESC",
                )
                .bind(organization_uuid)
                .bind(area_uuid)
                .bind(folder)
                .fetch_all(p)
                .await?
            } else {
                sqlx::query(
                    "SELECT uuid, organization_uuid, area_uuid, folder_uuid, title, short_summary, parent_page_uuid,
                     current_version_uuid, page_type, last_updated, created_at, auto_sync_to_vector_db,
                     vcs_export_allowed, includes_private_data, metadata
                     FROM module_docs_pages
                     WHERE organization_uuid = ?1 AND area_uuid = ?2 AND folder_uuid IS NULL
                     ORDER BY created_at DESC",
                )
                .bind(organization_uuid)
                .bind(area_uuid)
                .fetch_all(p)
                .await?
            };

            Ok(pages
                .into_iter()
                .map(|row| DocsPage {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    area_uuid: row.get("area_uuid"),
                    folder_uuid: row.get("folder_uuid"),
                    title: row.get("title"),
                    short_summary: row.get("short_summary"),
                    parent_page_uuid: row.get("parent_page_uuid"),
                    current_version_uuid: row.get("current_version_uuid"),
                    page_type: row.get("page_type"),
                    last_updated: row.get::<DateTime<Utc>, _>("last_updated"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    auto_sync_to_vector_db: row.get("auto_sync_to_vector_db"),
                    vcs_export_allowed: row.get("vcs_export_allowed"),
                    includes_private_data: row.get("includes_private_data"),
                    metadata: row.get("metadata"),
                })
                .collect())
        }
    }
}

/// Get all pages for a given organization and area
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `organization_uuid` - UUID of the organization
/// * `area_uuid` - UUID of the area
///
/// # Returns
/// Returns a vector of all pages sorted by created_at DESC
///
/// # Errors
/// Returns `DocsPageDatabaseError` if database operation fails
pub async fn get_all_pages(
    pool: &DatabasePool,
    organization_uuid: &str,
    area_uuid: &str,
) -> Result<Vec<DocsPage>, DocsPageDatabaseError> {
    match pool {
        DatabasePool::MySql(p) => {
            let pages = sqlx::query(
                "SELECT uuid, organization_uuid, area_uuid, folder_uuid, title, short_summary, parent_page_uuid,
                 current_version_uuid, page_type, last_updated, created_at, auto_sync_to_vector_db,
                 vcs_export_allowed, includes_private_data, metadata
                 FROM module_docs_pages
                 WHERE organization_uuid = ? AND area_uuid = ?
                 ORDER BY created_at DESC",
            )
            .bind(organization_uuid)
            .bind(area_uuid)
            .fetch_all(p)
            .await?;

            Ok(pages
                .into_iter()
                .map(|row| DocsPage {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    area_uuid: row.get("area_uuid"),
                    folder_uuid: row.get("folder_uuid"),
                    title: row.get("title"),
                    short_summary: row.get("short_summary"),
                    parent_page_uuid: row.get("parent_page_uuid"),
                    current_version_uuid: row.get("current_version_uuid"),
                    page_type: row.get("page_type"),
                    last_updated: row.get::<DateTime<Utc>, _>("last_updated"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    auto_sync_to_vector_db: row.get("auto_sync_to_vector_db"),
                    vcs_export_allowed: row.get("vcs_export_allowed"),
                    includes_private_data: row.get("includes_private_data"),
                    metadata: row.get("metadata"),
                })
                .collect())
        }
        DatabasePool::Postgres(p) => {
            let pages = sqlx::query(
                "SELECT uuid, organization_uuid, area_uuid, folder_uuid, title, short_summary, parent_page_uuid,
                 current_version_uuid, page_type, last_updated, created_at, auto_sync_to_vector_db,
                 vcs_export_allowed, includes_private_data, metadata
                 FROM module_docs_pages
                 WHERE organization_uuid = $1 AND area_uuid = $2
                 ORDER BY created_at DESC",
            )
            .bind(organization_uuid)
            .bind(area_uuid)
            .fetch_all(p)
            .await?;

            Ok(pages
                .into_iter()
                .map(|row| DocsPage {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    area_uuid: row.get("area_uuid"),
                    folder_uuid: row.get("folder_uuid"),
                    title: row.get("title"),
                    short_summary: row.get("short_summary"),
                    parent_page_uuid: row.get("parent_page_uuid"),
                    current_version_uuid: row.get("current_version_uuid"),
                    page_type: row.get("page_type"),
                    last_updated: row.get::<DateTime<Utc>, _>("last_updated"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    auto_sync_to_vector_db: row.get("auto_sync_to_vector_db"),
                    vcs_export_allowed: row.get("vcs_export_allowed"),
                    includes_private_data: row.get("includes_private_data"),
                    metadata: row.get("metadata"),
                })
                .collect())
        }
        DatabasePool::Sqlite(p) => {
            let pages = sqlx::query(
                "SELECT uuid, organization_uuid, area_uuid, folder_uuid, title, short_summary, parent_page_uuid,
                 current_version_uuid, page_type, last_updated, created_at, auto_sync_to_vector_db,
                 vcs_export_allowed, includes_private_data, metadata
                 FROM module_docs_pages
                 WHERE organization_uuid = ?1 AND area_uuid = ?2
                 ORDER BY created_at DESC",
            )
            .bind(organization_uuid)
            .bind(area_uuid)
            .fetch_all(p)
            .await?;

            Ok(pages
                .into_iter()
                .map(|row| DocsPage {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    area_uuid: row.get("area_uuid"),
                    folder_uuid: row.get("folder_uuid"),
                    title: row.get("title"),
                    short_summary: row.get("short_summary"),
                    parent_page_uuid: row.get("parent_page_uuid"),
                    current_version_uuid: row.get("current_version_uuid"),
                    page_type: row.get("page_type"),
                    last_updated: row.get::<DateTime<Utc>, _>("last_updated"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    auto_sync_to_vector_db: row.get("auto_sync_to_vector_db"),
                    vcs_export_allowed: row.get("vcs_export_allowed"),
                    includes_private_data: row.get("includes_private_data"),
                    metadata: row.get("metadata"),
                })
                .collect())
        }
    }
}

/// Load a page with its current version by page UUID
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `page_uuid` - UUID of the page
///
/// # Returns
/// Returns `DocsPageWithVersion` containing all page fields and the current version (if exists)
///
/// # Errors
/// Returns `DocsPageDatabaseError` if:
/// - Page not found
/// - Database operation fails
pub async fn load_page_with_version(
    pool: &DatabasePool,
    page_uuid: &str,
) -> Result<DocsPageWithVersion, DocsPageDatabaseError> {
    // First load the page
    let page = load_page_by_uuid(pool, page_uuid).await?;

    // Load the current version if it exists
    let version = if let Some(ref version_uuid) = page.current_version_uuid {
        match pool {
            DatabasePool::MySql(p) => {
                let row = sqlx::query(
                    "SELECT uuid, page_uuid, version_number, content, last_updated, created_at
                     FROM module_docs_page_versions WHERE uuid = ?",
                )
                .bind(version_uuid)
                .fetch_optional(p)
                .await?;

                row.map(|row| DocsPageVersion {
                    uuid: row.get("uuid"),
                    page_uuid: row.get("page_uuid"),
                    version_number: row.get("version_number"),
                    content: row.get("content"),
                    last_updated: row.get("last_updated"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                })
            }
            DatabasePool::Postgres(p) => {
                let row = sqlx::query(
                    "SELECT uuid, page_uuid, version_number, content, last_updated, created_at
                     FROM module_docs_page_versions WHERE uuid = $1",
                )
                .bind(version_uuid)
                .fetch_optional(p)
                .await?;

                row.map(|row| DocsPageVersion {
                    uuid: row.get("uuid"),
                    page_uuid: row.get("page_uuid"),
                    version_number: row.get("version_number"),
                    content: row.get("content"),
                    last_updated: row.get("last_updated"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                })
            }
            DatabasePool::Sqlite(p) => {
                let row = sqlx::query(
                    "SELECT uuid, page_uuid, version_number, content, last_updated, created_at
                     FROM module_docs_page_versions WHERE uuid = ?1",
                )
                .bind(version_uuid)
                .fetch_optional(p)
                .await?;

                row.map(|row| DocsPageVersion {
                    uuid: row.get("uuid"),
                    page_uuid: row.get("page_uuid"),
                    version_number: row.get("version_number"),
                    content: row.get("content"),
                    last_updated: row.get("last_updated"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                })
            }
        }
    } else {
        // If no current_version_uuid, try to get the latest version by version_number
        match pool {
            DatabasePool::MySql(p) => {
                let row = sqlx::query(
                    "SELECT uuid, page_uuid, version_number, content, last_updated, created_at
                     FROM module_docs_page_versions
                     WHERE page_uuid = ?
                     ORDER BY version_number DESC
                     LIMIT 1",
                )
                .bind(page_uuid)
                .fetch_optional(p)
                .await?;

                row.map(|row| DocsPageVersion {
                    uuid: row.get("uuid"),
                    page_uuid: row.get("page_uuid"),
                    version_number: row.get("version_number"),
                    content: row.get("content"),
                    last_updated: row.get("last_updated"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                })
            }
            DatabasePool::Postgres(p) => {
                let row = sqlx::query(
                    "SELECT uuid, page_uuid, version_number, content, last_updated, created_at
                     FROM module_docs_page_versions
                     WHERE page_uuid = $1
                     ORDER BY version_number DESC
                     LIMIT 1",
                )
                .bind(page_uuid)
                .fetch_optional(p)
                .await?;

                row.map(|row| DocsPageVersion {
                    uuid: row.get("uuid"),
                    page_uuid: row.get("page_uuid"),
                    version_number: row.get("version_number"),
                    content: row.get("content"),
                    last_updated: row.get("last_updated"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                })
            }
            DatabasePool::Sqlite(p) => {
                let row = sqlx::query(
                    "SELECT uuid, page_uuid, version_number, content, last_updated, created_at
                     FROM module_docs_page_versions
                     WHERE page_uuid = ?1
                     ORDER BY version_number DESC
                     LIMIT 1",
                )
                .bind(page_uuid)
                .fetch_optional(p)
                .await?;

                row.map(|row| DocsPageVersion {
                    uuid: row.get("uuid"),
                    page_uuid: row.get("page_uuid"),
                    version_number: row.get("version_number"),
                    content: row.get("content"),
                    last_updated: row.get("last_updated"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                })
            }
        }
    };

    Ok(DocsPageWithVersion {
        uuid: page.uuid,
        organization_uuid: page.organization_uuid,
        area_uuid: page.area_uuid,
        folder_uuid: page.folder_uuid,
        title: page.title,
        short_summary: page.short_summary,
        parent_page_uuid: page.parent_page_uuid,
        current_version_uuid: page.current_version_uuid,
        page_type: page.page_type,
        last_updated: page.last_updated,
        created_at: page.created_at,
        auto_sync_to_vector_db: page.auto_sync_to_vector_db,
        vcs_export_allowed: page.vcs_export_allowed,
        includes_private_data: page.includes_private_data,
        metadata: page.metadata,
        version,
    })
}

