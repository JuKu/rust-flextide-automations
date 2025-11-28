//! Docs Page module
//!
//! Provides functionality for managing documentation pages, including database operations and permission checks.

use chrono::{DateTime, Utc};
use flextide_core::database::{DatabaseError, DatabasePool};
use flextide_core::events::{Event, EventDispatcher, EventPayload};
use flextide_core::settings::{get_organizational_setting_value, SettingsDatabaseError};
use flextide_core::user::{user_belongs_to_organization, user_has_permission};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use sqlx::Row;
use thiserror::Error;
use tracing::{error, info, warn};

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

    #[error("Metadata must be a valid JSON object")]
    InvalidMetadata,

    #[error("Page version not found")]
    PageVersionNotFound,

    #[error("Settings error: {0}")]
    Settings(#[from] SettingsDatabaseError),

    #[error("AI provider setting not found or not configured")]
    AIProviderSettingNotFound,

    #[error("Unsupported AI provider: {0}")]
    UnsupportedAIProvider(String),

    #[error("Summary generation error: {0}")]
    SummaryGeneration(#[from] crate::summary::PageSummaryError),
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
    pub folder_uuid: Option<String>,
    pub parent_page_uuid: Option<String>,
    pub page_type: Option<String>,
    pub auto_sync_to_vector_db: Option<bool>,
    pub vcs_export_allowed: Option<bool>,
    pub includes_private_data: Option<bool>,
}

/// Request structure for moving a page
#[derive(Debug, Deserialize)]
pub struct MoveDocsPageRequest {
    pub folder_uuid: Option<String>,
    pub sort_order: i32,
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


/// Load a page by UUID and verify it belongs to the organization
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `page_uuid` - UUID of the page to load
/// * `organization_uuid` - UUID of the organization the page should belong to
///
/// # Returns
/// Returns the loaded page if it exists and belongs to the organization
///
/// # Errors
/// Returns `DocsPageDatabaseError` if:
/// - Page not found
/// - Page does not belong to the organization
/// - Database operation fails
async fn load_and_verify_page_ownership(
    pool: &DatabasePool,
    page_uuid: &str,
    organization_uuid: &str,
) -> Result<DocsPage, DocsPageDatabaseError> {
    let page = load_page_by_uuid(pool, page_uuid).await?;

    if page.organization_uuid != organization_uuid {
        warn!(
            "Page {} does not belong to organization {}",
            page_uuid, organization_uuid
        );
        return Err(DocsPageDatabaseError::PageNotInOrganization);
    }

    Ok(page)
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
    dispatcher: &EventDispatcher,
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
    // User must be a member of the area with can_add_pages permission, or be admin/owner
    let can_add = if let Some(perms) = &member_perms {
        perms.admin || perms.role == "owner" || perms.can_add_pages
    } else {
        // If not a member, user cannot add pages (even if area is public)
        false
    };

    // Also check for organization-wide super_admin permission
    let has_super_admin = user_has_permission(
        pool,
        user_uuid,
        organization_uuid,
        "super_admin",
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error checking super_admin permission: {}", e);
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
    })?;

    if !can_add && !has_super_admin {
        return Err(DocsPageDatabaseError::PermissionDenied);
    }

    // Determine flag values - inherit from folder if in a folder, otherwise use defaults
    // We already validated the folder above, so we can safely query for its flags
    let (auto_sync_to_vector_db, vcs_export_allowed, includes_private_data) = if let Some(ref folder_uuid) = request.folder_uuid {
        // Query folder flags (we already validated the folder exists above)
        let (folder_auto_sync, folder_vcs, folder_private) = match pool {
            DatabasePool::MySql(p) => {
                let row = sqlx::query(
                    "SELECT auto_sync_to_vector_db, vcs_export_allowed, includes_private_data
                     FROM module_docs_folders WHERE uuid = ?"
                )
                .bind(folder_uuid)
                .fetch_one(p)
                .await?;
                (
                    row.get::<i64, _>("auto_sync_to_vector_db") != 0,
                    row.get::<i64, _>("vcs_export_allowed") != 0,
                    row.get::<i64, _>("includes_private_data") != 0,
                )
            }
            DatabasePool::Postgres(p) => {
                let row = sqlx::query(
                    "SELECT auto_sync_to_vector_db, vcs_export_allowed, includes_private_data
                     FROM module_docs_folders WHERE uuid = $1"
                )
                .bind(folder_uuid)
                .fetch_one(p)
                .await?;
                (
                    row.get::<i32, _>("auto_sync_to_vector_db") != 0,
                    row.get::<i32, _>("vcs_export_allowed") != 0,
                    row.get::<i32, _>("includes_private_data") != 0,
                )
            }
            DatabasePool::Sqlite(p) => {
                let row = sqlx::query(
                    "SELECT auto_sync_to_vector_db, vcs_export_allowed, includes_private_data
                     FROM module_docs_folders WHERE uuid = ?1"
                )
                .bind(folder_uuid)
                .fetch_one(p)
                .await?;
                (
                    row.get::<i64, _>("auto_sync_to_vector_db") != 0,
                    row.get::<i64, _>("vcs_export_allowed") != 0,
                    row.get::<i64, _>("includes_private_data") != 0,
                )
            }
        };
        // Use folder's flags as defaults, but allow override from request
        (
            request.auto_sync_to_vector_db.unwrap_or(folder_auto_sync),
            request.vcs_export_allowed.unwrap_or(folder_vcs),
            request.includes_private_data.unwrap_or(folder_private),
        )
    } else {
        // Use request values or defaults
        (
            request.auto_sync_to_vector_db.unwrap_or(false),
            request.vcs_export_allowed.unwrap_or(false),
            request.includes_private_data.unwrap_or(false),
        )
    };

    // Create page
    let page_uuid = uuid::Uuid::new_v4().to_string();
    let page_type = request
        .page_type
        .unwrap_or_else(|| "markdown_page".to_string());

    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query(
                "INSERT INTO module_docs_pages (uuid, organization_uuid, area_uuid, folder_uuid, title, short_summary, parent_page_uuid, page_type, auto_sync_to_vector_db, vcs_export_allowed, includes_private_data)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&page_uuid)
            .bind(organization_uuid)
            .bind(&request.area_uuid)
            .bind(&request.folder_uuid)
            .bind(&request.title)
            .bind(&request.short_summary)
            .bind(&request.parent_page_uuid)
            .bind(&page_type)
            .bind(if auto_sync_to_vector_db { 1 } else { 0 })
            .bind(if vcs_export_allowed { 1 } else { 0 })
            .bind(if includes_private_data { 1 } else { 0 })
            .execute(p)
            .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO module_docs_pages (uuid, organization_uuid, area_uuid, folder_uuid, title, short_summary, parent_page_uuid, page_type, auto_sync_to_vector_db, vcs_export_allowed, includes_private_data)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
            )
            .bind(&page_uuid)
            .bind(organization_uuid)
            .bind(&request.area_uuid)
            .bind(&request.folder_uuid)
            .bind(&request.title)
            .bind(&request.short_summary)
            .bind(&request.parent_page_uuid)
            .bind(&page_type)
            .bind(if auto_sync_to_vector_db { 1 } else { 0 })
            .bind(if vcs_export_allowed { 1 } else { 0 })
            .bind(if includes_private_data { 1 } else { 0 })
            .execute(p)
            .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO module_docs_pages (uuid, organization_uuid, area_uuid, folder_uuid, title, short_summary, parent_page_uuid, page_type, auto_sync_to_vector_db, vcs_export_allowed, includes_private_data)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            )
            .bind(&page_uuid)
            .bind(organization_uuid)
            .bind(&request.area_uuid)
            .bind(&request.folder_uuid)
            .bind(&request.title)
            .bind(&request.short_summary)
            .bind(&request.parent_page_uuid)
            .bind(&page_type)
            .bind(if auto_sync_to_vector_db { 1 } else { 0 })
            .bind(if vcs_export_allowed { 1 } else { 0 })
            .bind(if includes_private_data { 1 } else { 0 })
            .execute(p)
            .await?;
        }
    }

    // Create initial version with template content (only for markdown_page type)
    if page_type == "markdown_page" {
        create_initial_page_version(pool, &page_uuid, &request.title).await?;
    }

    // Emit page created event
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

    dispatcher.emit(event).await;

    Ok(page_uuid)
}

/// Create an initial version for a page with template content
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `page_uuid` - UUID of the page
/// * `page_title` - Title of the page (used in template)
///
/// # Returns
/// Returns the UUID of the created version
///
/// # Errors
/// Returns `DocsPageDatabaseError` if database operation fails
async fn create_initial_page_version(
    pool: &DatabasePool,
    page_uuid: &str,
    page_title: &str,
) -> Result<String, DocsPageDatabaseError> {
    let version_uuid = uuid::Uuid::new_v4().to_string();
    let template_content = format!("# {}\n\n\n\n\n", page_title);
    let now = Utc::now();

    match pool {
        DatabasePool::MySql(p) => {
            // Create version
            sqlx::query(
                "INSERT INTO module_docs_page_versions (uuid, page_uuid, version_number, content, last_updated, created_at)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(&version_uuid)
            .bind(page_uuid)
            .bind(1) // First version
            .bind(&template_content)
            .bind(now)
            .bind(now)
            .execute(p)
            .await?;

            // Update page's current_version_uuid
            sqlx::query(
                "UPDATE module_docs_pages SET current_version_uuid = ?, last_updated = ? WHERE uuid = ?",
            )
            .bind(&version_uuid)
            .bind(now)
            .bind(page_uuid)
            .execute(p)
            .await?;
        }
        DatabasePool::Postgres(p) => {
            // Create version
            sqlx::query(
                "INSERT INTO module_docs_page_versions (uuid, page_uuid, version_number, content, last_updated, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(&version_uuid)
            .bind(page_uuid)
            .bind(1) // First version
            .bind(&template_content)
            .bind(now)
            .bind(now)
            .execute(p)
            .await?;

            // Update page's current_version_uuid
            sqlx::query(
                "UPDATE module_docs_pages SET current_version_uuid = $1, last_updated = $2 WHERE uuid = $3",
            )
            .bind(&version_uuid)
            .bind(now)
            .bind(page_uuid)
            .execute(p)
            .await?;
        }
        DatabasePool::Sqlite(p) => {
            // Create version
            sqlx::query(
                "INSERT INTO module_docs_page_versions (uuid, page_uuid, version_number, content, last_updated, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .bind(&version_uuid)
            .bind(page_uuid)
            .bind(1) // First version
            .bind(&template_content)
            .bind(now)
            .bind(now)
            .execute(p)
            .await?;

            // Update page's current_version_uuid
            sqlx::query(
                "UPDATE module_docs_pages SET current_version_uuid = ?1, last_updated = ?2 WHERE uuid = ?3",
            )
            .bind(&version_uuid)
            .bind(now)
            .bind(page_uuid)
            .execute(p)
            .await?;
        }
    }

    info!(
        "Created initial version {} for page {} with template content",
        version_uuid, page_uuid
    );

    Ok(version_uuid)
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
    dispatcher: &EventDispatcher,
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
    let page = load_and_verify_page_ownership(pool, page_uuid, organization_uuid).await?;

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

    dispatcher.emit(event).await;

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

/// Generate a summary for a documentation page using AI
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `organization_uuid` - UUID of the organization
/// * `page_uuid` - UUID of the page to generate summary for
/// * `dispatcher` - Optional event dispatcher to emit events
/// * `user_uuid` - Optional user UUID who triggered the generation
///
/// # Returns
/// Returns the generated summary as a String
///
/// # Errors
/// Returns `DocsPageDatabaseError` if:
/// - Page doesn't belong to the organization
/// - Page not found
/// - Page version not found
/// - AI provider setting not configured
/// - Unsupported AI provider
/// - Summary generation fails
pub async fn generate_page_summary(
    pool: &DatabasePool,
    organization_uuid: &str,
    page_uuid: &str,
    dispatcher: &EventDispatcher,
    user_uuid: Option<&str>,
) -> Result<String, DocsPageDatabaseError> {
    info!(
        "Starting summary generation for page {} in organization {}",
        page_uuid, organization_uuid
    );

    // Load the page and verify it belongs to the organization
    let page = load_and_verify_page_ownership(pool, page_uuid, organization_uuid).await?;

    info!("Page {} belongs to organization {}", page_uuid, organization_uuid);

    // Get the current version
    let version_uuid = page.current_version_uuid.clone().ok_or_else(|| {
        error!("Page {} has no current version", page_uuid);
        DocsPageDatabaseError::PageVersionNotFound
    })?;

    info!("Loading page version {} for page {}", version_uuid, page_uuid);

    // Load the version
    let version = match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query(
                "SELECT uuid, page_uuid, version_number, content, last_updated, created_at
                 FROM module_docs_page_versions WHERE uuid = ?",
            )
            .bind(&version_uuid)
            .fetch_optional(p)
            .await?;

            match row {
                Some(row) => DocsPageVersion {
                    uuid: row.get("uuid"),
                    page_uuid: row.get("page_uuid"),
                    version_number: row.get("version_number"),
                    content: row.get("content"),
                    last_updated: row.get("last_updated"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                },
                None => {
                    error!("Version {} not found for page {}", version_uuid, page_uuid);
                    return Err(DocsPageDatabaseError::PageVersionNotFound);
                }
            }
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT uuid, page_uuid, version_number, content, last_updated, created_at
                 FROM module_docs_page_versions WHERE uuid = $1",
            )
            .bind(&version_uuid)
            .fetch_optional(p)
            .await?;

            match row {
                Some(row) => DocsPageVersion {
                    uuid: row.get("uuid"),
                    page_uuid: row.get("page_uuid"),
                    version_number: row.get("version_number"),
                    content: row.get("content"),
                    last_updated: row.get("last_updated"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                },
                None => {
                    error!("Version {} not found for page {}", version_uuid, page_uuid);
                    return Err(DocsPageDatabaseError::PageVersionNotFound);
                }
            }
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT uuid, page_uuid, version_number, content, last_updated, created_at
                 FROM module_docs_page_versions WHERE uuid = ?1",
            )
            .bind(&version_uuid)
            .fetch_optional(p)
            .await?;

            match row {
                Some(row) => DocsPageVersion {
                    uuid: row.get("uuid"),
                    page_uuid: row.get("page_uuid"),
                    version_number: row.get("version_number"),
                    content: row.get("content"),
                    last_updated: row.get("last_updated"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                },
                None => {
                    error!("Version {} not found for page {}", version_uuid, page_uuid);
                    return Err(DocsPageDatabaseError::PageVersionNotFound);
                }
            }
        }
    };

    info!(
        "Page version {} loaded successfully (content length: {} characters)",
        version_uuid,
        version.content.len()
    );

    // Get the AI provider setting
    let ai_provider = get_organizational_setting_value(
        pool,
        organization_uuid,
        "module_docs_page_summary_ai_provider",
    )
    .await?;

    let ai_provider = ai_provider.ok_or_else(|| {
        error!(
            "AI provider setting not configured for organization {}",
            organization_uuid
        );
        DocsPageDatabaseError::AIProviderSettingNotFound
    })?;

    info!(
        "Using AI provider '{}' for summary generation",
        ai_provider
    );

    // Create the appropriate generator based on the provider
    let generator: Box<dyn crate::summary::PageSummaryGenerator> = match ai_provider.as_str() {
        "openai" => {
            // Get OpenAI API key from settings
            let api_key = get_organizational_setting_value(
                pool,
                organization_uuid,
                "module_docs_openai_api_key",
            )
            .await?
            .ok_or_else(|| {
                error!("OpenAI API key not configured for organization {}", organization_uuid);
                DocsPageDatabaseError::AIProviderSettingNotFound
            })?;

            // Get OpenAI model from settings (default to gpt-4o-mini if not set)
            let model = get_organizational_setting_value(
                pool,
                organization_uuid,
                "module_docs_openai_model",
            )
            .await?
            .unwrap_or_else(|| "gpt-4o-mini".to_string());

            info!("Creating OpenAI generator with model: {}", model);
            Box::new(crate::summary::OpenAIPageSummaryGenerator::new(api_key, model))
        }
        "claude" => {
            error!("Claude provider not yet implemented");
            return Err(DocsPageDatabaseError::UnsupportedAIProvider(ai_provider));
        }
        "gemini" => {
            error!("Gemini provider not yet implemented");
            return Err(DocsPageDatabaseError::UnsupportedAIProvider(ai_provider));
        }
        _ => {
            error!("Unsupported AI provider: {}", ai_provider);
            return Err(DocsPageDatabaseError::UnsupportedAIProvider(ai_provider));
        }
    };

    // Generate the summary
    info!(
        "Calling AI provider '{}' to generate summary for page {}",
        ai_provider, page_uuid
    );

    let summary = generator.generate_summary(&page, &version).await?;

    info!(
        "Successfully generated summary for page {} (length: {} characters)",
        page_uuid,
        summary.len()
    );

    // Emit page summary generated event
    let mut event = Event::new(
        "module_docs_page_summary_generated",
        EventPayload::new(json!({
            "entity_type": "page",
            "entity_id": page_uuid,
            "organization_uuid": organization_uuid,
            "data": {
                "page_uuid": page_uuid,
                "summary_length": summary.len(),
                "ai_provider": ai_provider
            }
        })),
    )
    .with_organization(organization_uuid);
    
    if let Some(user_uuid) = user_uuid {
        event = event.with_user(user_uuid);
    }
    
    dispatcher.emit(event).await;

    Ok(summary)
}

/// Save a summary for a documentation page
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `organization_uuid` - UUID of the organization
/// * `page_uuid` - UUID of the page to save summary for
/// * `summary` - The summary text to save
/// * `dispatcher` - Optional event dispatcher to emit events
/// * `user_uuid` - Optional user UUID who saved the summary
///
/// # Returns
/// Returns `()` on success
///
/// # Errors
/// Returns `DocsPageDatabaseError` if:
/// - Page doesn't belong to the organization
/// - Page not found
/// - Database operation fails
pub async fn save_page_summary(
    pool: &DatabasePool,
    organization_uuid: &str,
    page_uuid: &str,
    summary: &str,
    dispatcher: &EventDispatcher,
    user_uuid: Option<&str>,
) -> Result<(), DocsPageDatabaseError> {
    info!(
        "Saving summary for page {} in organization {}",
        page_uuid, organization_uuid
    );

    // Load the page and verify it belongs to the organization
    load_and_verify_page_ownership(pool, page_uuid, organization_uuid).await?;

    info!("Page {} belongs to organization {}", page_uuid, organization_uuid);

    // Update the short_summary field
    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query(
                "UPDATE module_docs_pages SET short_summary = ? WHERE uuid = ? AND organization_uuid = ?",
            )
            .bind(summary)
            .bind(page_uuid)
            .bind(organization_uuid)
            .execute(p)
            .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query(
                "UPDATE module_docs_pages SET short_summary = $1 WHERE uuid = $2 AND organization_uuid = $3",
            )
            .bind(summary)
            .bind(page_uuid)
            .bind(organization_uuid)
            .execute(p)
            .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query(
                "UPDATE module_docs_pages SET short_summary = ?1 WHERE uuid = ?2 AND organization_uuid = ?3",
            )
            .bind(summary)
            .bind(page_uuid)
            .bind(organization_uuid)
            .execute(p)
            .await?;
        }
    }

    info!(
        "Successfully saved summary for page {} (length: {} characters)",
        page_uuid,
        summary.len()
    );

    // Emit page summary updated event
    let mut event = Event::new(
        "module_docs_page_summary_updated",
        EventPayload::new(json!({
            "entity_type": "page",
            "entity_id": page_uuid,
            "organization_uuid": organization_uuid,
            "data": {
                "page_uuid": page_uuid,
                "summary_length": summary.len()
            }
        })),
    )
    .with_organization(organization_uuid);
    
    if let Some(user_uuid) = user_uuid {
        event = event.with_user(user_uuid);
    }
    
    dispatcher.emit(event).await;

    Ok(())
}

/// Save page content by creating a new version (if content changed)
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `organization_uuid` - UUID of the organization
/// * `page_uuid` - UUID of the page to save content for
/// * `user_uuid` - UUID of the user saving the content
/// * `content` - The content text to save
/// * `dispatcher` - Optional event dispatcher to emit events
///
/// # Returns
/// Returns the UUID of the new version (or existing version if content unchanged)
///
/// # Errors
/// Returns `DocsPageDatabaseError` if:
/// - Page doesn't belong to the organization
/// - Page not found
/// - User doesn't have permission to edit pages
/// - Database operation fails
pub async fn save_page_content(
    pool: &DatabasePool,
    organization_uuid: &str,
    page_uuid: &str,
    user_uuid: &str,
    content: &str,
    dispatcher: &EventDispatcher,
) -> Result<String, DocsPageDatabaseError> {
    use flextide_core::user::user_has_permission;
    use crate::area::{load_area_by_uuid, load_area_member_permissions};

    info!(
        "Saving content for page {} in organization {}",
        page_uuid, organization_uuid
    );

    // Load the page and verify it belongs to the organization
    let page = load_and_verify_page_ownership(pool, page_uuid, organization_uuid).await?;

    info!("Page {} belongs to organization {}", page_uuid, organization_uuid);

    // Verify area exists (for permission checking)
    let _area = load_area_by_uuid(pool, &page.area_uuid)
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

    // Check if user has permission to edit pages
    let can_edit = if let Some(perms) = &member_perms {
        perms.admin || perms.role == "owner" || perms.can_edit_pages
        // TODO: Add check for can_edit_own_pages when creator_uuid is added to pages
    } else {
        // If not a member, user cannot edit pages
        false
    };

    // Also check for organization-wide super_admin permission
    let has_super_admin = user_has_permission(
        pool,
        user_uuid,
        organization_uuid,
        "module_docs_super_admin",
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
    })?;

    if !can_edit && !has_super_admin {
        warn!(
            "User {} does not have permission to edit page {}",
            user_uuid, page_uuid
        );
        return Err(DocsPageDatabaseError::PermissionDenied);
    }

    // Get current version content if it exists
    let current_content = if let Some(ref version_uuid) = page.current_version_uuid {
        match pool {
            DatabasePool::MySql(p) => {
                let row = sqlx::query(
                    "SELECT content FROM module_docs_page_versions WHERE uuid = ?",
                )
                .bind(version_uuid)
                .fetch_optional(p)
                .await?;

                row.map(|row| row.get::<String, _>("content"))
            }
            DatabasePool::Postgres(p) => {
                let row = sqlx::query(
                    "SELECT content FROM module_docs_page_versions WHERE uuid = $1",
                )
                .bind(version_uuid)
                .fetch_optional(p)
                .await?;

                row.map(|row| row.get::<String, _>("content"))
            }
            DatabasePool::Sqlite(p) => {
                let row = sqlx::query(
                    "SELECT content FROM module_docs_page_versions WHERE uuid = ?1",
                )
                .bind(version_uuid)
                .fetch_optional(p)
                .await?;

                row.map(|row| row.get::<String, _>("content"))
            }
        }
    } else {
        None
    };

    // Check if content has changed
    let content_changed = current_content.as_ref().map(|c| c != content).unwrap_or(true);

    if !content_changed {
        info!("Content for page {} unchanged, returning existing version", page_uuid);
        return Ok(page.current_version_uuid.unwrap());
    }

    // Get the next version number
    let next_version_number = match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query(
                "SELECT COALESCE(MAX(version_number), 0) + 1 as next_version
                 FROM module_docs_page_versions WHERE page_uuid = ?",
            )
            .bind(page_uuid)
            .fetch_one(p)
            .await?;

            row.get::<i32, _>("next_version")
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT COALESCE(MAX(version_number), 0) + 1 as next_version
                 FROM module_docs_page_versions WHERE page_uuid = $1",
            )
            .bind(page_uuid)
            .fetch_one(p)
            .await?;

            row.get::<i32, _>("next_version")
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT COALESCE(MAX(version_number), 0) + 1 as next_version
                 FROM module_docs_page_versions WHERE page_uuid = ?1",
            )
            .bind(page_uuid)
            .fetch_one(p)
            .await?;

            row.get::<i32, _>("next_version")
        }
    };

    // Create new version
    let version_uuid = uuid::Uuid::new_v4().to_string();
    let now = Utc::now();

    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query(
                "INSERT INTO module_docs_page_versions (uuid, page_uuid, version_number, content, last_updated, created_at)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(&version_uuid)
            .bind(page_uuid)
            .bind(next_version_number)
            .bind(content)
            .bind(now)
            .bind(now)
            .execute(p)
            .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO module_docs_page_versions (uuid, page_uuid, version_number, content, last_updated, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(&version_uuid)
            .bind(page_uuid)
            .bind(next_version_number)
            .bind(content)
            .bind(now)
            .bind(now)
            .execute(p)
            .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO module_docs_page_versions (uuid, page_uuid, version_number, content, last_updated, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .bind(&version_uuid)
            .bind(page_uuid)
            .bind(next_version_number)
            .bind(content)
            .bind(now)
            .bind(now)
            .execute(p)
            .await?;
        }
    }

    // Update page's current_version_uuid and last_updated
    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query(
                "UPDATE module_docs_pages SET current_version_uuid = ?, last_updated = ? WHERE uuid = ? AND organization_uuid = ?",
            )
            .bind(&version_uuid)
            .bind(now)
            .bind(page_uuid)
            .bind(organization_uuid)
            .execute(p)
            .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query(
                "UPDATE module_docs_pages SET current_version_uuid = $1, last_updated = $2 WHERE uuid = $3 AND organization_uuid = $4",
            )
            .bind(&version_uuid)
            .bind(now)
            .bind(page_uuid)
            .bind(organization_uuid)
            .execute(p)
            .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query(
                "UPDATE module_docs_pages SET current_version_uuid = ?1, last_updated = ?2 WHERE uuid = ?3 AND organization_uuid = ?4",
            )
            .bind(&version_uuid)
            .bind(now)
            .bind(page_uuid)
            .bind(organization_uuid)
            .execute(p)
            .await?;
        }
    }

    info!(
        "Successfully saved content for page {} (version {}, length: {} characters)",
        page_uuid,
        next_version_number,
        content.len()
    );

    // Emit page version created event
    let event = Event::new(
        "module_docs_page_version_created",
        EventPayload::new(json!({
            "entity_type": "page_version",
            "entity_id": version_uuid,
            "organization_uuid": organization_uuid,
            "data": {
                "page_uuid": page_uuid,
                "version_uuid": version_uuid,
                "version_number": next_version_number,
                "content_length": content.len()
            }
        })),
    )
    .with_organization(organization_uuid)
    .with_user(user_uuid);

    dispatcher.emit(event).await;

    Ok(version_uuid)
}

/// List page versions with pagination
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `page_uuid` - UUID of the page
/// * `limit` - Maximum number of versions to return (default: 15)
/// * `offset` - Number of versions to skip (default: 0)
///
/// # Returns
/// Vector of `DocsPageVersion` structs ordered by version_number DESC
///
/// # Errors
/// Returns `DocsPageDatabaseError` if database operation fails
pub async fn list_page_versions(
    pool: &DatabasePool,
    page_uuid: &str,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<DocsPageVersion>, DocsPageDatabaseError> {
    let limit = limit.unwrap_or(15);
    let offset = offset.unwrap_or(0);

    match pool {
        DatabasePool::MySql(p) => {
            let rows = sqlx::query(
                "SELECT uuid, page_uuid, version_number, content, last_updated, created_at
                 FROM module_docs_page_versions
                 WHERE page_uuid = ?
                 ORDER BY version_number DESC
                 LIMIT ? OFFSET ?",
            )
            .bind(page_uuid)
            .bind(limit)
            .bind(offset)
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .map(|row| DocsPageVersion {
                    uuid: row.get("uuid"),
                    page_uuid: row.get("page_uuid"),
                    version_number: row.get("version_number"),
                    content: row.get("content"),
                    last_updated: row.get("last_updated"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                })
                .collect())
        }
        DatabasePool::Postgres(p) => {
            let rows = sqlx::query(
                "SELECT uuid, page_uuid, version_number, content, last_updated, created_at
                 FROM module_docs_page_versions
                 WHERE page_uuid = $1
                 ORDER BY version_number DESC
                 LIMIT $2 OFFSET $3",
            )
            .bind(page_uuid)
            .bind(limit)
            .bind(offset)
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .map(|row| DocsPageVersion {
                    uuid: row.get("uuid"),
                    page_uuid: row.get("page_uuid"),
                    version_number: row.get("version_number"),
                    content: row.get("content"),
                    last_updated: row.get("last_updated"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                })
                .collect())
        }
        DatabasePool::Sqlite(p) => {
            let rows = sqlx::query(
                "SELECT uuid, page_uuid, version_number, content, last_updated, created_at
                 FROM module_docs_page_versions
                 WHERE page_uuid = ?1
                 ORDER BY version_number DESC
                 LIMIT ?2 OFFSET ?3",
            )
            .bind(page_uuid)
            .bind(limit)
            .bind(offset)
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .map(|row| DocsPageVersion {
                    uuid: row.get("uuid"),
                    page_uuid: row.get("page_uuid"),
                    version_number: row.get("version_number"),
                    content: row.get("content"),
                    last_updated: row.get("last_updated"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                })
                .collect())
        }
    }
}

/// Update page properties (title, short_summary, auto_sync_to_vector_db, vcs_export_allowed, includes_private_data, metadata)
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `page_uuid` - UUID of the page to update
/// * `organization_uuid` - UUID of the organization
/// * `user_uuid` - UUID of the user updating the page
/// * `title` - New title for the page
/// * `short_summary` - New short summary (optional)
/// * `auto_sync_to_vector_db` - Whether to auto-sync to vector DB
/// * `vcs_export_allowed` - Whether VCS export is allowed
/// * `includes_private_data` - Whether page includes private data
/// * `metadata` - JSON metadata (must be a valid JSON object)
/// * `dispatcher` - Optional event dispatcher
///
/// # Returns
/// Returns `()` on success
///
/// # Errors
/// Returns `DocsPageDatabaseError` if:
/// - User does not belong to the organization
/// - User does not have permission to edit page properties
/// - Page does not belong to the organization
/// - Title is empty
/// - Metadata is not a valid JSON object
/// - Database operation fails
pub async fn update_page_properties(
    pool: &DatabasePool,
    page_uuid: &str,
    organization_uuid: &str,
    user_uuid: &str,
    title: &str,
    short_summary: Option<&str>,
    auto_sync_to_vector_db: bool,
    vcs_export_allowed: bool,
    includes_private_data: bool,
    metadata: serde_json::Value,
    dispatcher: &EventDispatcher,
) -> Result<(), DocsPageDatabaseError> {
    // Validate title is not empty
    if title.trim().is_empty() {
        return Err(DocsPageDatabaseError::EmptyTitle);
    }

    // Validate metadata is a JSON object (not array or primitive)
    if !metadata.is_object() {
        return Err(DocsPageDatabaseError::InvalidMetadata);
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

    // Load page to verify it belongs to the organization
    let page = load_and_verify_page_ownership(pool, page_uuid, organization_uuid).await?;

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

    // Check if user has permission to edit page properties
    let can_edit = if let Some(perms) = &member_perms {
        perms.admin || perms.role == "owner" || perms.can_edit_pages
    } else {
        false
    };

    // Also check for organization-wide super_admin permission
    let has_super_admin = user_has_permission(
        pool,
        user_uuid,
        organization_uuid,
        "module_docs_super_admin",
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error checking super_admin permission: {}", e);
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
    })?;

    if !can_edit && !has_super_admin {
        warn!(
            "User {} does not have permission to edit page properties {}",
            user_uuid, page_uuid
        );
        return Err(DocsPageDatabaseError::PermissionDenied);
    }

    // Update page properties
    match pool {
        DatabasePool::MySql(p) => {
            let result = sqlx::query(
                "UPDATE module_docs_pages SET title = ?, short_summary = ?, auto_sync_to_vector_db = ?, vcs_export_allowed = ?, includes_private_data = ?, metadata = ? WHERE uuid = ? AND organization_uuid = ?"
            )
                .bind(title.trim())
                .bind(short_summary.map(|s| s.trim()).unwrap_or(""))
                .bind(if auto_sync_to_vector_db { 1 } else { 0 })
                .bind(if vcs_export_allowed { 1 } else { 0 })
                .bind(if includes_private_data { 1 } else { 0 })
                .bind(&metadata)
                .bind(page_uuid)
                .bind(organization_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsPageDatabaseError::PageNotFound);
            }
        }
        DatabasePool::Postgres(p) => {
            let result = sqlx::query(
                "UPDATE module_docs_pages SET title = $1, short_summary = $2, auto_sync_to_vector_db = $3, vcs_export_allowed = $4, includes_private_data = $5, metadata = $6 WHERE uuid = $7 AND organization_uuid = $8"
            )
                .bind(title.trim())
                .bind(short_summary.map(|s| s.trim()).unwrap_or(""))
                .bind(if auto_sync_to_vector_db { 1 } else { 0 })
                .bind(if vcs_export_allowed { 1 } else { 0 })
                .bind(if includes_private_data { 1 } else { 0 })
                .bind(&metadata)
                .bind(page_uuid)
                .bind(organization_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsPageDatabaseError::PageNotFound);
            }
        }
        DatabasePool::Sqlite(p) => {
            let result = sqlx::query(
                "UPDATE module_docs_pages SET title = ?1, short_summary = ?2, auto_sync_to_vector_db = ?3, vcs_export_allowed = ?4, includes_private_data = ?5, metadata = ?6 WHERE uuid = ?7 AND organization_uuid = ?8"
            )
                .bind(title.trim())
                .bind(short_summary.map(|s| s.trim()).unwrap_or(""))
                .bind(if auto_sync_to_vector_db { 1 } else { 0 })
                .bind(if vcs_export_allowed { 1 } else { 0 })
                .bind(if includes_private_data { 1 } else { 0 })
                .bind(&metadata)
                .bind(page_uuid)
                .bind(organization_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsPageDatabaseError::PageNotFound);
            }
        }
    }

    // Emit page properties updated event
    let page = load_page_by_uuid(pool, page_uuid).await.ok();
    let event = Event::new(
        "module_docs_page_properties_updated",
        EventPayload::new(json!({
            "entity_type": "page",
            "entity_id": page_uuid,
            "organization_uuid": organization_uuid,
            "data": page.as_ref().map(|p| json!({
                "title": p.title,
                "short_summary": p.short_summary,
                "auto_sync_to_vector_db": p.auto_sync_to_vector_db,
                "vcs_export_allowed": p.vcs_export_allowed,
                "includes_private_data": p.includes_private_data,
                "metadata": p.metadata
            })).unwrap_or(json!({}))
        }))
    )
    .with_organization(organization_uuid)
    .with_user(user_uuid);

    dispatcher.emit(event).await;

    info!(
        "Successfully updated properties for page {} in organization {}",
        page_uuid, organization_uuid
    );

    Ok(())
}

/// Move a page to a different folder
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `page_uuid` - UUID of the page to move
/// * `organization_uuid` - UUID of the organization
/// * `user_uuid` - UUID of the user performing the move
/// * `folder_uuid` - UUID of the target folder (None for root)
/// * `sort_order` - Sort order (currently not used, but kept for API consistency)
/// * `dispatcher` - Optional event dispatcher
///
/// # Returns
/// * `Ok(())` if the page was moved successfully
/// * `Err(DocsPageDatabaseError)` if an error occurred
pub async fn move_page(
    pool: &DatabasePool,
    page_uuid: &str,
    organization_uuid: &str,
    user_uuid: &str,
    folder_uuid: Option<String>,
    _sort_order: i32,
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
    let page = load_and_verify_page_ownership(pool, page_uuid, organization_uuid).await?;

    // Validate folder if provided
    if let Some(ref folder_uuid_str) = folder_uuid {
        use crate::folder::load_folder_by_uuid;
        use crate::folder::DocsFolderDatabaseError as FolderError;
        
        let folder = load_folder_by_uuid(pool, folder_uuid_str)
            .await
            .map_err(|e| match e {
                FolderError::Database(db_err) => DocsPageDatabaseError::Database(db_err),
                FolderError::Sql(sql_err) => DocsPageDatabaseError::Sql(sql_err),
                FolderError::FolderNotFound => DocsPageDatabaseError::PageNotFound, // Reuse error type
                _ => DocsPageDatabaseError::Database(
                    flextide_core::database::DatabaseError::PoolCreationFailed(
                        sqlx::Error::RowNotFound,
                    ),
                ),
            })?;
        
        if folder.organization_uuid != organization_uuid {
            return Err(DocsPageDatabaseError::PageNotInOrganization);
        }
        
        if folder.area_uuid != page.area_uuid {
            return Err(DocsPageDatabaseError::AreaNotInOrganization);
        }
    }

    // Load area to check permissions
    let _area = load_area_by_uuid(pool, &page.area_uuid)
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

    // Check if user has permission to edit pages
    let can_edit = if let Some(perms) = &member_perms {
        perms.admin || perms.role == "owner" || perms.can_edit_pages
    } else {
        false
    };

    // Also check for organization-wide super_admin permission
    let has_super_admin = user_has_permission(
        pool,
        user_uuid,
        organization_uuid,
        "module_docs_super_admin",
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error checking super_admin permission: {}", e);
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
    })?;

    if !can_edit && !has_super_admin {
        return Err(DocsPageDatabaseError::PermissionDenied);
    }

    // Update folder_uuid
    match pool {
        DatabasePool::MySql(p) => {
            let result = sqlx::query(
                "UPDATE module_docs_pages SET folder_uuid = ? WHERE uuid = ? AND organization_uuid = ?"
            )
                .bind(&folder_uuid)
                .bind(page_uuid)
                .bind(organization_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsPageDatabaseError::PageNotFound);
            }
        }
        DatabasePool::Postgres(p) => {
            let result = sqlx::query(
                "UPDATE module_docs_pages SET folder_uuid = $1 WHERE uuid = $2 AND organization_uuid = $3"
            )
                .bind(&folder_uuid)
                .bind(page_uuid)
                .bind(organization_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsPageDatabaseError::PageNotFound);
            }
        }
        DatabasePool::Sqlite(p) => {
            let result = sqlx::query(
                "UPDATE module_docs_pages SET folder_uuid = ?1 WHERE uuid = ?2 AND organization_uuid = ?3"
            )
                .bind(&folder_uuid)
                .bind(page_uuid)
                .bind(organization_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsPageDatabaseError::PageNotFound);
            }
        }
    }

    // Emit page moved event
    if let Some(disp) = dispatcher {
        let page = load_page_by_uuid(pool, page_uuid).await.ok();
        let event = Event::new(
            "module_docs_page_moved",
            EventPayload::new(json!({
                "entity_type": "page",
                "entity_id": page_uuid,
                "organization_uuid": organization_uuid,
                "data": page.as_ref().map(|p| json!({
                    "title": p.title,
                    "folder_uuid": p.folder_uuid,
                    "area_uuid": p.area_uuid
                })).unwrap_or(json!({}))
            }))
        )
        .with_organization(organization_uuid)
        .with_user(user_uuid);

        disp.emit(event).await;
    }

    info!(
        "Successfully moved page {} to folder {:?} in organization {}",
        page_uuid, folder_uuid, organization_uuid
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn test_create_initial_page_version(pool: sqlx::SqlitePool) -> sqlx::Result<()> {
        let pool = DatabasePool::Sqlite(pool);

        // Set up required tables
        match &pool {
            DatabasePool::Sqlite(p) => {
                // Create organizations table
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS organizations (
                        uuid CHAR(36) NOT NULL PRIMARY KEY,
                        name VARCHAR(255) NOT NULL,
                        owner_user_id CHAR(36) NOT NULL,
                        created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
                    )"
                )
                .execute(p)
                .await
                .expect("Failed to create organizations table");

                // Create module_docs_areas table
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS module_docs_areas (
                        uuid CHAR(36) NOT NULL PRIMARY KEY,
                        organization_uuid CHAR(36) NOT NULL,
                        short_name VARCHAR(255) NOT NULL,
                        description TEXT,
                        icon_name VARCHAR(50),
                        color_hex VARCHAR(20),
                        topics TEXT,
                        public INTEGER NOT NULL DEFAULT 0,
                        visible INTEGER NOT NULL DEFAULT 1,
                        deletable INTEGER NOT NULL DEFAULT 1,
                        creator_uuid CHAR(36) NOT NULL,
                        created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        FOREIGN KEY (organization_uuid) REFERENCES organizations(uuid) ON DELETE CASCADE
                    )"
                )
                .execute(p)
                .await
                .expect("Failed to create module_docs_areas table");

                // Create module_docs_pages table
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS module_docs_pages (
                        uuid CHAR(36) NOT NULL PRIMARY KEY,
                        organization_uuid CHAR(36) NOT NULL,
                        area_uuid CHAR(36) NOT NULL,
                        folder_uuid CHAR(36),
                        title VARCHAR(255) NOT NULL,
                        short_summary TEXT,
                        parent_page_uuid CHAR(36),
                        current_version_uuid CHAR(36),
                        page_type VARCHAR(50) NOT NULL DEFAULT 'markdown_page',
                        last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        auto_sync_to_vector_db INTEGER NOT NULL DEFAULT 0,
                        vcs_export_allowed INTEGER NOT NULL DEFAULT 0,
                        includes_private_data INTEGER NOT NULL DEFAULT 0,
                        metadata TEXT,
                        FOREIGN KEY (organization_uuid) REFERENCES organizations(uuid) ON DELETE CASCADE,
                        FOREIGN KEY (area_uuid) REFERENCES module_docs_areas(uuid) ON DELETE CASCADE
                    )"
                )
                .execute(p)
                .await
                .expect("Failed to create module_docs_pages table");

                // Create module_docs_page_versions table
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS module_docs_page_versions (
                        uuid CHAR(36) NOT NULL PRIMARY KEY,
                        page_uuid CHAR(36) NOT NULL,
                        version_number INTEGER NOT NULL DEFAULT 1,
                        content TEXT NOT NULL,
                        last_updated TIMESTAMP,
                        created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        FOREIGN KEY (page_uuid) REFERENCES module_docs_pages(uuid) ON DELETE CASCADE,
                        CONSTRAINT unique_page_version UNIQUE (page_uuid, version_number)
                    )"
                )
                .execute(p)
                .await
                .expect("Failed to create module_docs_page_versions table");
            }
            _ => panic!("Test only supports SQLite"),
        }

        // Create test data
        let org_uuid = uuid::Uuid::new_v4().to_string();
        let area_uuid = uuid::Uuid::new_v4().to_string();
        let page_uuid = uuid::Uuid::new_v4().to_string();
        let user_uuid = uuid::Uuid::new_v4().to_string();
        let page_title = "Test Page Title";

        match &pool {
            DatabasePool::Sqlite(p) => {
                // Insert organization
                sqlx::query(
                    "INSERT INTO organizations (uuid, name, owner_user_id) VALUES (?1, ?2, ?3)"
                )
                .bind(&org_uuid)
                .bind("Test Org")
                .bind(&user_uuid)
                .execute(p)
                .await
                .expect("Failed to insert organization");

                // Insert area
                sqlx::query(
                    "INSERT INTO module_docs_areas (uuid, organization_uuid, short_name, creator_uuid, public, visible, deletable)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
                )
                .bind(&area_uuid)
                .bind(&org_uuid)
                .bind("Test Area")
                .bind(&user_uuid)
                .bind(0)
                .bind(1)
                .bind(1)
                .execute(p)
                .await
                .expect("Failed to insert area");

                // Insert page
                sqlx::query(
                    "INSERT INTO module_docs_pages (uuid, organization_uuid, area_uuid, title, page_type, auto_sync_to_vector_db, vcs_export_allowed, includes_private_data)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
                )
                .bind(&page_uuid)
                .bind(&org_uuid)
                .bind(&area_uuid)
                .bind(page_title)
                .bind("markdown_page")
                .bind(0)
                .bind(0)
                .bind(0)
                .execute(p)
                .await
                .expect("Failed to insert page");
            }
            _ => unreachable!(),
        }

        // Test: Create initial version
        let version_uuid = create_initial_page_version(&pool, &page_uuid, page_title)
            .await
            .expect("Failed to create initial page version");

        // Verify version was created
        match &pool {
            DatabasePool::Sqlite(p) => {
                // Check version exists with correct content
                let version_row = sqlx::query(
                    "SELECT uuid, page_uuid, version_number, content FROM module_docs_page_versions WHERE uuid = ?1"
                )
                .bind(&version_uuid)
                .fetch_one(p)
                .await
                .expect("Version should exist");

                assert_eq!(version_row.get::<String, _>("uuid"), version_uuid);
                assert_eq!(version_row.get::<String, _>("page_uuid"), page_uuid);
                assert_eq!(version_row.get::<i32, _>("version_number"), 1);
                
                let expected_content = format!("# {}\n\n\n\n\n", page_title);
                assert_eq!(version_row.get::<String, _>("content"), expected_content);

                // Check page's current_version_uuid was updated
                let page_row = sqlx::query(
                    "SELECT current_version_uuid FROM module_docs_pages WHERE uuid = ?1"
                )
                .bind(&page_uuid)
                .fetch_one(p)
                .await
                .expect("Page should exist");

                let current_version: Option<String> = page_row.get("current_version_uuid");
                assert_eq!(current_version, Some(version_uuid.clone()));
            }
            _ => unreachable!(),
        }

        Ok(())
    }

    #[sqlx::test]
    async fn test_list_page_versions(pool: sqlx::SqlitePool) -> sqlx::Result<()> {
        let pool = DatabasePool::Sqlite(pool);

        // Set up required tables
        match &pool {
            DatabasePool::Sqlite(p) => {
                // Create organizations table
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS organizations (
                        uuid CHAR(36) NOT NULL PRIMARY KEY,
                        name VARCHAR(255) NOT NULL,
                        owner_user_id CHAR(36) NOT NULL,
                        created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
                    )"
                )
                .execute(p)
                .await
                .expect("Failed to create organizations table");

                // Create module_docs_areas table
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS module_docs_areas (
                        uuid CHAR(36) NOT NULL PRIMARY KEY,
                        organization_uuid CHAR(36) NOT NULL,
                        short_name VARCHAR(255) NOT NULL,
                        description TEXT,
                        icon_name VARCHAR(50),
                        color_hex VARCHAR(20),
                        topics TEXT,
                        public INTEGER NOT NULL DEFAULT 0,
                        visible INTEGER NOT NULL DEFAULT 1,
                        deletable INTEGER NOT NULL DEFAULT 1,
                        creator_uuid CHAR(36) NOT NULL,
                        created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        FOREIGN KEY (organization_uuid) REFERENCES organizations(uuid) ON DELETE CASCADE
                    )"
                )
                .execute(p)
                .await
                .expect("Failed to create module_docs_areas table");

                // Create module_docs_pages table
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS module_docs_pages (
                        uuid CHAR(36) NOT NULL PRIMARY KEY,
                        organization_uuid CHAR(36) NOT NULL,
                        area_uuid CHAR(36) NOT NULL,
                        folder_uuid CHAR(36),
                        title VARCHAR(255) NOT NULL,
                        short_summary TEXT,
                        parent_page_uuid CHAR(36),
                        current_version_uuid CHAR(36),
                        page_type VARCHAR(50) NOT NULL DEFAULT 'markdown_page',
                        last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        auto_sync_to_vector_db INTEGER NOT NULL DEFAULT 0,
                        vcs_export_allowed INTEGER NOT NULL DEFAULT 0,
                        includes_private_data INTEGER NOT NULL DEFAULT 0,
                        metadata TEXT,
                        FOREIGN KEY (organization_uuid) REFERENCES organizations(uuid) ON DELETE CASCADE,
                        FOREIGN KEY (area_uuid) REFERENCES module_docs_areas(uuid) ON DELETE CASCADE
                    )"
                )
                .execute(p)
                .await
                .expect("Failed to create module_docs_pages table");

                // Create module_docs_page_versions table
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS module_docs_page_versions (
                        uuid CHAR(36) NOT NULL PRIMARY KEY,
                        page_uuid CHAR(36) NOT NULL,
                        version_number INTEGER NOT NULL DEFAULT 1,
                        content TEXT NOT NULL,
                        last_updated TIMESTAMP,
                        created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        FOREIGN KEY (page_uuid) REFERENCES module_docs_pages(uuid) ON DELETE CASCADE,
                        CONSTRAINT unique_page_version UNIQUE (page_uuid, version_number)
                    )"
                )
                .execute(p)
                .await
                .expect("Failed to create module_docs_page_versions table");
            }
            _ => panic!("Test only supports SQLite"),
        }

        // Create test data
        let org_uuid = uuid::Uuid::new_v4().to_string();
        let area_uuid = uuid::Uuid::new_v4().to_string();
        let page_uuid = uuid::Uuid::new_v4().to_string();
        let other_page_uuid = uuid::Uuid::new_v4().to_string();
        let user_uuid = uuid::Uuid::new_v4().to_string();

        match &pool {
            DatabasePool::Sqlite(p) => {
                // Insert organization
                sqlx::query(
                    "INSERT INTO organizations (uuid, name, owner_user_id) VALUES (?1, ?2, ?3)"
                )
                .bind(&org_uuid)
                .bind("Test Org")
                .bind(&user_uuid)
                .execute(p)
                .await
                .expect("Failed to insert organization");

                // Insert area
                sqlx::query(
                    "INSERT INTO module_docs_areas (uuid, organization_uuid, short_name, creator_uuid, public, visible, deletable)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
                )
                .bind(&area_uuid)
                .bind(&org_uuid)
                .bind("Test Area")
                .bind(&user_uuid)
                .bind(0)
                .bind(1)
                .bind(1)
                .execute(p)
                .await
                .expect("Failed to insert area");

                // Insert pages
                sqlx::query(
                    "INSERT INTO module_docs_pages (uuid, organization_uuid, area_uuid, title, page_type, auto_sync_to_vector_db, vcs_export_allowed, includes_private_data)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
                )
                .bind(&page_uuid)
                .bind(&org_uuid)
                .bind(&area_uuid)
                .bind("Test Page")
                .bind("markdown_page")
                .bind(0)
                .bind(0)
                .bind(0)
                .execute(p)
                .await
                .expect("Failed to insert page");

                sqlx::query(
                    "INSERT INTO module_docs_pages (uuid, organization_uuid, area_uuid, title, page_type, auto_sync_to_vector_db, vcs_export_allowed, includes_private_data)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
                )
                .bind(&other_page_uuid)
                .bind(&org_uuid)
                .bind(&area_uuid)
                .bind("Other Page")
                .bind("markdown_page")
                .bind(0)
                .bind(0)
                .bind(0)
                .execute(p)
                .await
                .expect("Failed to insert other page");

                // Create multiple versions for the test page with different creation dates
                // We'll insert them with explicit timestamps to ensure proper ordering
                let now = chrono::Utc::now();
                let versions_data = vec![
                    (1, "Version 1 content", now - chrono::Duration::days(5)),
                    (2, "Version 2 content", now - chrono::Duration::days(3)),
                    (3, "Version 3 content", now - chrono::Duration::days(1)),
                    (4, "Version 4 content", now),
                ];

                for (version_num, content, created_at) in versions_data {
                    let version_uuid = uuid::Uuid::new_v4().to_string();
                    sqlx::query(
                        "INSERT INTO module_docs_page_versions (uuid, page_uuid, version_number, content, created_at)
                         VALUES (?1, ?2, ?3, ?4, ?5)"
                    )
                    .bind(&version_uuid)
                    .bind(&page_uuid)
                    .bind(version_num)
                    .bind(content)
                    .bind(created_at)
                    .execute(p)
                    .await
                    .expect(&format!("Failed to insert version {}", version_num));
                }

                // Create a version for the other page to ensure filtering works
                let other_version_uuid = uuid::Uuid::new_v4().to_string();
                sqlx::query(
                    "INSERT INTO module_docs_page_versions (uuid, page_uuid, version_number, content, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)"
                )
                .bind(&other_version_uuid)
                .bind(&other_page_uuid)
                .bind(1)
                .bind("Other page content")
                .bind(now)
                .execute(p)
                .await
                .expect("Failed to insert other page version");
            }
            _ => unreachable!(),
        }

        // Test 1: List all versions (default limit)
        let versions = list_page_versions(&pool, &page_uuid, None, None)
            .await
            .expect("Failed to list page versions");

        assert_eq!(versions.len(), 4, "Should return all 4 versions");
        
        // Verify ordering by version_number DESC (highest version number first)
        for i in 0..versions.len() - 1 {
            assert!(
                versions[i].version_number >= versions[i + 1].version_number,
                "Versions should be ordered by version_number DESC (highest first)"
            );
        }

        // Verify all versions belong to the correct page
        for version in &versions {
            assert_eq!(version.page_uuid, page_uuid, "Version should belong to test page");
        }

        // Verify version numbers match expected order (highest version number first should be version 4)
        assert_eq!(versions[0].version_number, 4, "First version should be version 4 (highest)");
        assert_eq!(versions[3].version_number, 1, "Last version should be version 1 (lowest)");

        // Test 2: Pagination with limit
        let limited_versions = list_page_versions(&pool, &page_uuid, Some(2), None)
            .await
            .expect("Failed to list page versions with limit");

        assert_eq!(limited_versions.len(), 2, "Should return only 2 versions");
        assert_eq!(limited_versions[0].version_number, 4, "First should be version 4");
        assert_eq!(limited_versions[1].version_number, 3, "Second should be version 3");

        // Test 3: Pagination with offset
        let offset_versions = list_page_versions(&pool, &page_uuid, Some(2), Some(2))
            .await
            .expect("Failed to list page versions with offset");

        assert_eq!(offset_versions.len(), 2, "Should return 2 versions");
        assert_eq!(offset_versions[0].version_number, 2, "First should be version 2");
        assert_eq!(offset_versions[1].version_number, 1, "Second should be version 1");

        // Test 4: Empty result for non-existent page
        let non_existent_uuid = uuid::Uuid::new_v4().to_string();
        let empty_versions = list_page_versions(&pool, &non_existent_uuid, None, None)
            .await
            .expect("Should not error for non-existent page");

        assert_eq!(empty_versions.len(), 0, "Should return empty list for non-existent page");

        // Test 5: Verify other page's versions are not included
        let all_versions = list_page_versions(&pool, &page_uuid, None, None)
            .await
            .expect("Failed to list page versions");

        for version in &all_versions {
            assert_ne!(version.page_uuid, other_page_uuid, "Should not include other page's versions");
        }

        Ok(())
    }
}

