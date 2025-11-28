//! Docs Folder module
//!
//! Provides functionality for managing documentation folders, including database operations and permission checks.

use chrono::{DateTime, Utc};
use flextide_core::database::{DatabaseError, DatabasePool};
use flextide_core::events::{Event, EventDispatcher, EventPayload};
use flextide_core::user::{user_belongs_to_organization, user_has_permission};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use thiserror::Error;

use crate::area::{
    load_area_by_uuid, load_area_member_permissions, DocsAreaDatabaseError,
};

/// Error type for Docs folder database operations
#[derive(Debug, Error)]
pub enum DocsFolderDatabaseError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("SQL execution error: {0}")]
    Sql(#[from] sqlx::Error),

    #[error("User does not belong to this organization")]
    UserNotInOrganization,

    #[error("User does not have permission to perform this action")]
    PermissionDenied,

    #[error("Folder not found")]
    FolderNotFound,

    #[error("Folder does not belong to this organization")]
    FolderNotInOrganization,

    #[error("Area not found")]
    AreaNotFound,

    #[error("Area does not belong to this organization")]
    AreaNotInOrganization,

    #[error("Name cannot be empty")]
    EmptyName,

    #[error("Folder cannot be deleted: contains sub-folders or pages")]
    FolderNotEmpty,

    #[error("Metadata must be a valid JSON object")]
    InvalidMetadata,
}

/// Docs Folder data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsFolder {
    pub uuid: String,
    pub organization_uuid: String,
    pub area_uuid: String,
    pub name: String,
    pub icon_name: Option<String>,
    pub folder_color: Option<String>,
    pub parent_folder_uuid: Option<String>,
    pub sort_order: i32,
    pub visible: bool,
    pub created_at: DateTime<Utc>,
    pub activated: bool,
    pub auto_sync_to_vector_db: bool,
    pub vcs_export_allowed: bool,
    pub includes_private_data: bool,
    pub metadata: Option<serde_json::Value>,
}

/// Request structure for creating a new folder
#[derive(Debug, Deserialize)]
pub struct CreateDocsFolderRequest {
    pub area_uuid: String,
    pub name: String,
    pub icon_name: Option<String>,
    pub folder_color: Option<String>,
    pub parent_folder_uuid: Option<String>,
    pub sort_order: Option<i32>,
}

/// Request structure for updating a folder
#[derive(Debug, Deserialize)]
pub struct UpdateDocsFolderRequest {
    pub name: Option<String>,
    pub icon_name: Option<String>,
    pub folder_color: Option<String>,
    pub sort_order: Option<i32>,
}

/// Request structure for moving a folder
#[derive(Debug, Deserialize)]
pub struct MoveDocsFolderRequest {
    pub parent_folder_uuid: Option<String>,
    pub sort_order: i32,
}

/// Load folders for a given organization and area, optionally filtered by parent folder
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `organization_uuid` - UUID of the organization
/// * `area_uuid` - UUID of the area
/// * `parent_folder_uuid` - Optional parent folder UUID (None for root folders)
///
/// # Returns
/// Returns a vector of folders sorted by sort_order ASC
///
/// # Errors
/// Returns `DocsFolderDatabaseError` if database operation fails
pub async fn list_folders(
    pool: &DatabasePool,
    organization_uuid: &str,
    area_uuid: &str,
    parent_folder_uuid: Option<&str>,
) -> Result<Vec<DocsFolder>, DocsFolderDatabaseError> {
    match pool {
        DatabasePool::MySql(p) => {
            let folders = if let Some(parent_uuid) = parent_folder_uuid {
                sqlx::query(
                    "SELECT uuid, organization_uuid, area_uuid, name, icon_name, folder_color, parent_folder_uuid,
                     sort_order, visible, created_at, activated, auto_sync_to_vector_db, vcs_export_allowed,
                     includes_private_data, metadata
                     FROM module_docs_folders
                     WHERE organization_uuid = ? AND area_uuid = ? AND parent_folder_uuid = ?
                     ORDER BY sort_order ASC",
                )
                .bind(organization_uuid)
                .bind(area_uuid)
                .bind(parent_uuid)
                .fetch_all(p)
                .await?
            } else {
                sqlx::query(
                    "SELECT uuid, organization_uuid, area_uuid, name, icon_name, folder_color, parent_folder_uuid,
                     sort_order, visible, created_at, activated, auto_sync_to_vector_db, vcs_export_allowed,
                     includes_private_data, metadata
                     FROM module_docs_folders
                     WHERE organization_uuid = ? AND area_uuid = ? AND parent_folder_uuid IS NULL
                     ORDER BY sort_order ASC",
                )
                .bind(organization_uuid)
                .bind(area_uuid)
                .fetch_all(p)
                .await?
            };

            Ok(folders
                .into_iter()
                .map(|row| DocsFolder {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    area_uuid: row.get("area_uuid"),
                    name: row.get("name"),
                    icon_name: row.get("icon_name"),
                    folder_color: row.get("folder_color"),
                    parent_folder_uuid: row.get("parent_folder_uuid"),
                    sort_order: row.get("sort_order"),
                    visible: row.get::<i64, _>("visible") != 0,
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    activated: row.get::<i64, _>("activated") != 0,
                    auto_sync_to_vector_db: row.get::<i64, _>("auto_sync_to_vector_db") != 0,
                    vcs_export_allowed: row.get::<i64, _>("vcs_export_allowed") != 0,
                    includes_private_data: row.get::<i64, _>("includes_private_data") != 0,
                    metadata: row.get::<Option<serde_json::Value>, _>("metadata"),
                })
                .collect())
        }
        DatabasePool::Postgres(p) => {
            let folders = if let Some(parent_uuid) = parent_folder_uuid {
                sqlx::query(
                    "SELECT uuid, organization_uuid, area_uuid, name, icon_name, folder_color, parent_folder_uuid,
                     sort_order, visible, created_at, activated, auto_sync_to_vector_db, vcs_export_allowed,
                     includes_private_data, metadata
                     FROM module_docs_folders
                     WHERE organization_uuid = $1 AND area_uuid = $2 AND parent_folder_uuid = $3
                     ORDER BY sort_order ASC",
                )
                .bind(organization_uuid)
                .bind(area_uuid)
                .bind(parent_uuid)
                .fetch_all(p)
                .await?
            } else {
                sqlx::query(
                    "SELECT uuid, organization_uuid, area_uuid, name, icon_name, folder_color, parent_folder_uuid,
                     sort_order, visible, created_at, activated, auto_sync_to_vector_db, vcs_export_allowed,
                     includes_private_data, metadata
                     FROM module_docs_folders
                     WHERE organization_uuid = $1 AND area_uuid = $2 AND parent_folder_uuid IS NULL
                     ORDER BY sort_order ASC",
                )
                .bind(organization_uuid)
                .bind(area_uuid)
                .fetch_all(p)
                .await?
            };

            Ok(folders
                .into_iter()
                .map(|row| DocsFolder {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    area_uuid: row.get("area_uuid"),
                    name: row.get("name"),
                    icon_name: row.get("icon_name"),
                    folder_color: row.get("folder_color"),
                    parent_folder_uuid: row.get("parent_folder_uuid"),
                    sort_order: row.get("sort_order"),
                    visible: row.get::<i32, _>("visible") != 0,
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    activated: row.get::<i32, _>("activated") != 0,
                    auto_sync_to_vector_db: row.get::<i32, _>("auto_sync_to_vector_db") != 0,
                    vcs_export_allowed: row.get::<i32, _>("vcs_export_allowed") != 0,
                    includes_private_data: row.get::<i32, _>("includes_private_data") != 0,
                    metadata: row.get::<Option<serde_json::Value>, _>("metadata"),
                })
                .collect())
        }
        DatabasePool::Sqlite(p) => {
            let folders = if let Some(parent_uuid) = parent_folder_uuid {
                sqlx::query(
                    "SELECT uuid, organization_uuid, area_uuid, name, icon_name, folder_color, parent_folder_uuid,
                     sort_order, visible, created_at, activated, auto_sync_to_vector_db, vcs_export_allowed,
                     includes_private_data, metadata
                     FROM module_docs_folders
                     WHERE organization_uuid = ?1 AND area_uuid = ?2 AND parent_folder_uuid = ?3
                     ORDER BY sort_order ASC",
                )
                .bind(organization_uuid)
                .bind(area_uuid)
                .bind(parent_uuid)
                .fetch_all(p)
                .await?
            } else {
                sqlx::query(
                    "SELECT uuid, organization_uuid, area_uuid, name, icon_name, folder_color, parent_folder_uuid,
                     sort_order, visible, created_at, activated, auto_sync_to_vector_db, vcs_export_allowed,
                     includes_private_data, metadata
                     FROM module_docs_folders
                     WHERE organization_uuid = ?1 AND area_uuid = ?2 AND parent_folder_uuid IS NULL
                     ORDER BY sort_order ASC",
                )
                .bind(organization_uuid)
                .bind(area_uuid)
                .fetch_all(p)
                .await?
            };

            Ok(folders
                .into_iter()
                .map(|row| DocsFolder {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    area_uuid: row.get("area_uuid"),
                    name: row.get("name"),
                    icon_name: row.get("icon_name"),
                    folder_color: row.get("folder_color"),
                    parent_folder_uuid: row.get("parent_folder_uuid"),
                    sort_order: row.get("sort_order"),
                    visible: row.get::<i64, _>("visible") != 0,
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    activated: row.get::<i64, _>("activated") != 0,
                    auto_sync_to_vector_db: row.get::<i64, _>("auto_sync_to_vector_db") != 0,
                    vcs_export_allowed: row.get::<i64, _>("vcs_export_allowed") != 0,
                    includes_private_data: row.get::<i64, _>("includes_private_data") != 0,
                    metadata: row.get::<Option<serde_json::Value>, _>("metadata"),
                })
                .collect())
        }
    }
}

/// Get all folders for a given organization and area
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `organization_uuid` - UUID of the organization
/// * `area_uuid` - UUID of the area
///
/// # Returns
/// Returns a vector of all folders sorted by sort_order ASC
///
/// # Errors
/// Returns `DocsFolderDatabaseError` if database operation fails
pub async fn get_all_folders(
    pool: &DatabasePool,
    organization_uuid: &str,
    area_uuid: &str,
) -> Result<Vec<DocsFolder>, DocsFolderDatabaseError> {
    match pool {
        DatabasePool::MySql(p) => {
            let folders = sqlx::query(
                "SELECT uuid, organization_uuid, area_uuid, name, icon_name, folder_color, parent_folder_uuid,
                 sort_order, visible, created_at, activated, auto_sync_to_vector_db, vcs_export_allowed,
                 includes_private_data, metadata
                 FROM module_docs_folders
                 WHERE organization_uuid = ? AND area_uuid = ?
                 ORDER BY sort_order ASC",
            )
            .bind(organization_uuid)
            .bind(area_uuid)
            .fetch_all(p)
            .await?;

            Ok(folders
                .into_iter()
                .map(|row| DocsFolder {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    area_uuid: row.get("area_uuid"),
                    name: row.get("name"),
                    icon_name: row.get("icon_name"),
                    folder_color: row.get("folder_color"),
                    parent_folder_uuid: row.get("parent_folder_uuid"),
                    sort_order: row.get("sort_order"),
                    visible: row.get::<i64, _>("visible") != 0,
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    activated: row.get::<i64, _>("activated") != 0,
                    auto_sync_to_vector_db: row.get::<i64, _>("auto_sync_to_vector_db") != 0,
                    vcs_export_allowed: row.get::<i64, _>("vcs_export_allowed") != 0,
                    includes_private_data: row.get::<i64, _>("includes_private_data") != 0,
                    metadata: row.get::<Option<serde_json::Value>, _>("metadata"),
                })
                .collect())
        }
        DatabasePool::Postgres(p) => {
            let folders = sqlx::query(
                "SELECT uuid, organization_uuid, area_uuid, name, icon_name, folder_color, parent_folder_uuid,
                 sort_order, visible, created_at, activated, auto_sync_to_vector_db, vcs_export_allowed,
                 includes_private_data, metadata
                 FROM module_docs_folders
                 WHERE organization_uuid = $1 AND area_uuid = $2
                 ORDER BY sort_order ASC",
            )
            .bind(organization_uuid)
            .bind(area_uuid)
            .fetch_all(p)
            .await?;

            Ok(folders
                .into_iter()
                .map(|row| DocsFolder {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    area_uuid: row.get("area_uuid"),
                    name: row.get("name"),
                    icon_name: row.get("icon_name"),
                    folder_color: row.get("folder_color"),
                    parent_folder_uuid: row.get("parent_folder_uuid"),
                    sort_order: row.get("sort_order"),
                    visible: row.get::<i32, _>("visible") != 0,
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    activated: row.get::<i32, _>("activated") != 0,
                    auto_sync_to_vector_db: row.get::<i32, _>("auto_sync_to_vector_db") != 0,
                    vcs_export_allowed: row.get::<i32, _>("vcs_export_allowed") != 0,
                    includes_private_data: row.get::<i32, _>("includes_private_data") != 0,
                    metadata: row.get::<Option<serde_json::Value>, _>("metadata"),
                })
                .collect())
        }
        DatabasePool::Sqlite(p) => {
            let folders = sqlx::query(
                "SELECT uuid, organization_uuid, area_uuid, name, icon_name, folder_color, parent_folder_uuid,
                 sort_order, visible, created_at, activated, auto_sync_to_vector_db, vcs_export_allowed,
                 includes_private_data, metadata
                 FROM module_docs_folders
                 WHERE organization_uuid = ?1 AND area_uuid = ?2
                 ORDER BY sort_order ASC",
            )
            .bind(organization_uuid)
            .bind(area_uuid)
            .fetch_all(p)
            .await?;

            Ok(folders
                .into_iter()
                .map(|row| DocsFolder {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    area_uuid: row.get("area_uuid"),
                    name: row.get("name"),
                    icon_name: row.get("icon_name"),
                    folder_color: row.get("folder_color"),
                    parent_folder_uuid: row.get("parent_folder_uuid"),
                    sort_order: row.get("sort_order"),
                    visible: row.get::<i64, _>("visible") != 0,
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    activated: row.get::<i64, _>("activated") != 0,
                    auto_sync_to_vector_db: row.get::<i64, _>("auto_sync_to_vector_db") != 0,
                    vcs_export_allowed: row.get::<i64, _>("vcs_export_allowed") != 0,
                    includes_private_data: row.get::<i64, _>("includes_private_data") != 0,
                    metadata: row.get::<Option<serde_json::Value>, _>("metadata"),
                })
                .collect())
        }
    }
}

/// Load a folder by UUID
pub async fn load_folder_by_uuid(
    pool: &DatabasePool,
    folder_uuid: &str,
) -> Result<DocsFolder, DocsFolderDatabaseError> {
    match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query(
                "SELECT uuid, organization_uuid, area_uuid, name, icon_name, folder_color, parent_folder_uuid,
                 sort_order, visible, created_at, activated, auto_sync_to_vector_db, vcs_export_allowed,
                 includes_private_data, metadata
                 FROM module_docs_folders WHERE uuid = ?",
            )
            .bind(folder_uuid)
            .fetch_optional(p)
            .await?;

            match row {
                Some(row) => Ok(DocsFolder {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    area_uuid: row.get("area_uuid"),
                    name: row.get("name"),
                    icon_name: row.get("icon_name"),
                    folder_color: row.get("folder_color"),
                    parent_folder_uuid: row.get("parent_folder_uuid"),
                    sort_order: row.get("sort_order"),
                    visible: row.get::<i64, _>("visible") != 0,
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    activated: row.get::<i64, _>("activated") != 0,
                    auto_sync_to_vector_db: row.get::<i64, _>("auto_sync_to_vector_db") != 0,
                    vcs_export_allowed: row.get::<i64, _>("vcs_export_allowed") != 0,
                    includes_private_data: row.get::<i64, _>("includes_private_data") != 0,
                    metadata: row.get::<Option<serde_json::Value>, _>("metadata"),
                }),
                None => Err(DocsFolderDatabaseError::FolderNotFound),
            }
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT uuid, organization_uuid, area_uuid, name, icon_name, folder_color, parent_folder_uuid,
                 sort_order, visible, created_at, activated, auto_sync_to_vector_db, vcs_export_allowed,
                 includes_private_data, metadata
                 FROM module_docs_folders WHERE uuid = $1",
            )
            .bind(folder_uuid)
            .fetch_optional(p)
            .await?;

            match row {
                Some(row) => Ok(DocsFolder {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    area_uuid: row.get("area_uuid"),
                    name: row.get("name"),
                    icon_name: row.get("icon_name"),
                    folder_color: row.get("folder_color"),
                    parent_folder_uuid: row.get("parent_folder_uuid"),
                    sort_order: row.get("sort_order"),
                    visible: row.get::<i32, _>("visible") != 0,
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    activated: row.get::<i32, _>("activated") != 0,
                    auto_sync_to_vector_db: row.get::<i32, _>("auto_sync_to_vector_db") != 0,
                    vcs_export_allowed: row.get::<i32, _>("vcs_export_allowed") != 0,
                    includes_private_data: row.get::<i32, _>("includes_private_data") != 0,
                    metadata: row.get::<Option<serde_json::Value>, _>("metadata"),
                }),
                None => Err(DocsFolderDatabaseError::FolderNotFound),
            }
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT uuid, organization_uuid, area_uuid, name, icon_name, folder_color, parent_folder_uuid,
                 sort_order, visible, created_at, activated, auto_sync_to_vector_db, vcs_export_allowed,
                 includes_private_data, metadata
                 FROM module_docs_folders WHERE uuid = ?1",
            )
            .bind(folder_uuid)
            .fetch_optional(p)
            .await?;

            match row {
                Some(row) => Ok(DocsFolder {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    area_uuid: row.get("area_uuid"),
                    name: row.get("name"),
                    icon_name: row.get("icon_name"),
                    folder_color: row.get("folder_color"),
                    parent_folder_uuid: row.get("parent_folder_uuid"),
                    sort_order: row.get("sort_order"),
                    visible: row.get::<i64, _>("visible") != 0,
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    activated: row.get::<i64, _>("activated") != 0,
                    auto_sync_to_vector_db: row.get::<i64, _>("auto_sync_to_vector_db") != 0,
                    vcs_export_allowed: row.get::<i64, _>("vcs_export_allowed") != 0,
                    includes_private_data: row.get::<i64, _>("includes_private_data") != 0,
                    metadata: row.get::<Option<serde_json::Value>, _>("metadata"),
                }),
                None => Err(DocsFolderDatabaseError::FolderNotFound),
            }
        }
    }
}

/// Create a new folder in the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `organization_uuid` - UUID of the organization
/// * `user_uuid` - UUID of the user creating the folder
/// * `request` - Folder creation request
///
/// # Returns
/// Returns the UUID of the newly created folder
///
/// # Errors
/// Returns `DocsFolderDatabaseError` if:
/// - User does not belong to the organization
/// - User does not have permission to create folders
/// - Area does not belong to the organization
/// - Name is empty
/// - Database operation fails
pub async fn create_folder(
    pool: &DatabasePool,
    organization_uuid: &str,
    user_uuid: &str,
    request: CreateDocsFolderRequest,
    dispatcher: Option<&EventDispatcher>,
) -> Result<String, DocsFolderDatabaseError> {
    // Validate name
    if request.name.trim().is_empty() {
        return Err(DocsFolderDatabaseError::EmptyName);
    }

    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(pool, user_uuid, organization_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            DocsFolderDatabaseError::Database(e.into())
        })?;

    if !belongs {
        return Err(DocsFolderDatabaseError::UserNotInOrganization);
    }

    // Load area to verify it belongs to the organization
    let area = load_area_by_uuid(pool, &request.area_uuid)
        .await
        .map_err(|e| match e {
            DocsAreaDatabaseError::AreaNotFound => DocsFolderDatabaseError::AreaNotFound,
            DocsAreaDatabaseError::Database(e) => DocsFolderDatabaseError::Database(e),
            DocsAreaDatabaseError::Sql(e) => DocsFolderDatabaseError::Sql(e),
            DocsAreaDatabaseError::UserNotInOrganization => {
                DocsFolderDatabaseError::UserNotInOrganization
            }
            DocsAreaDatabaseError::PermissionDenied => DocsFolderDatabaseError::PermissionDenied,
            DocsAreaDatabaseError::AreaNotInOrganization => {
                DocsFolderDatabaseError::AreaNotInOrganization
            }
            DocsAreaDatabaseError::EmptyShortName => {
                DocsFolderDatabaseError::Database(
                    flextide_core::database::DatabaseError::PoolCreationFailed(
                        sqlx::Error::RowNotFound,
                    ),
                )
            }
        })?;

    if area.organization_uuid != organization_uuid {
        return Err(DocsFolderDatabaseError::AreaNotInOrganization);
    }

    // Check area member permissions
    let member_perms = load_area_member_permissions(pool, &request.area_uuid, user_uuid)
        .await
        .map_err(|e| match e {
            DocsAreaDatabaseError::Database(e) => DocsFolderDatabaseError::Database(e),
            DocsAreaDatabaseError::Sql(e) => DocsFolderDatabaseError::Sql(e),
            DocsAreaDatabaseError::UserNotInOrganization => {
                DocsFolderDatabaseError::UserNotInOrganization
            }
            DocsAreaDatabaseError::PermissionDenied => DocsFolderDatabaseError::PermissionDenied,
            DocsAreaDatabaseError::AreaNotFound => DocsFolderDatabaseError::AreaNotFound,
            DocsAreaDatabaseError::AreaNotInOrganization => {
                DocsFolderDatabaseError::AreaNotInOrganization
            }
            DocsAreaDatabaseError::EmptyShortName => {
                DocsFolderDatabaseError::Database(
                    flextide_core::database::DatabaseError::PoolCreationFailed(
                        sqlx::Error::RowNotFound,
                    ),
                )
            }
        })?;

    // Check if user has permission to add folders
    let can_add = if let Some(perms) = &member_perms {
        // Check if user is a member and has can_add_folders permission
        perms.admin || perms.role == "owner" || perms.can_add_folders
    } else {
        // If not a member, user cannot add folders
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
                DocsFolderDatabaseError::Database(db_err)
            }
            flextide_core::user::UserDatabaseError::Sql(sql_err) => {
                DocsFolderDatabaseError::Sql(sql_err)
            }
            _ => DocsFolderDatabaseError::Database(
                flextide_core::database::DatabaseError::PoolCreationFailed(
                    sqlx::Error::RowNotFound,
                ),
            ),
        }
    })?;

    if !can_add && !has_super_admin {
        return Err(DocsFolderDatabaseError::PermissionDenied);
    }

    // Validate parent folder if provided and inherit properties
    let (auto_sync_to_vector_db, vcs_export_allowed, includes_private_data) = if let Some(ref parent_uuid) = request.parent_folder_uuid {
        let parent_folder = load_folder_by_uuid(pool, parent_uuid).await?;
        if parent_folder.organization_uuid != organization_uuid {
            return Err(DocsFolderDatabaseError::FolderNotInOrganization);
        }
        if parent_folder.area_uuid != request.area_uuid {
            return Err(DocsFolderDatabaseError::AreaNotInOrganization);
        }
        // Inherit properties from parent (except metadata)
        (parent_folder.auto_sync_to_vector_db, parent_folder.vcs_export_allowed, parent_folder.includes_private_data)
    } else {
        // Default values if no parent
        (false, false, false)
    };

    // Create folder
    let folder_uuid = uuid::Uuid::new_v4().to_string();
    let sort_order = request.sort_order.unwrap_or(0);

    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query(
                "INSERT INTO module_docs_folders (uuid, organization_uuid, area_uuid, name, icon_name, folder_color, parent_folder_uuid, sort_order, auto_sync_to_vector_db, vcs_export_allowed, includes_private_data, metadata)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&folder_uuid)
            .bind(organization_uuid)
            .bind(&request.area_uuid)
            .bind(&request.name)
            .bind(&request.icon_name)
            .bind(&request.folder_color)
            .bind(&request.parent_folder_uuid)
            .bind(sort_order)
            .bind(if auto_sync_to_vector_db { 1 } else { 0 })
            .bind(if vcs_export_allowed { 1 } else { 0 })
            .bind(if includes_private_data { 1 } else { 0 })
            .bind(serde_json::json!({}))
            .execute(p)
            .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO module_docs_folders (uuid, organization_uuid, area_uuid, name, icon_name, folder_color, parent_folder_uuid, sort_order, auto_sync_to_vector_db, vcs_export_allowed, includes_private_data, metadata)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
            )
            .bind(&folder_uuid)
            .bind(organization_uuid)
            .bind(&request.area_uuid)
            .bind(&request.name)
            .bind(&request.icon_name)
            .bind(&request.folder_color)
            .bind(&request.parent_folder_uuid)
            .bind(sort_order)
            .bind(if auto_sync_to_vector_db { 1 } else { 0 })
            .bind(if vcs_export_allowed { 1 } else { 0 })
            .bind(if includes_private_data { 1 } else { 0 })
            .bind(serde_json::json!({}))
            .execute(p)
            .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO module_docs_folders (uuid, organization_uuid, area_uuid, name, icon_name, folder_color, parent_folder_uuid, sort_order, auto_sync_to_vector_db, vcs_export_allowed, includes_private_data, metadata)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            )
            .bind(&folder_uuid)
            .bind(organization_uuid)
            .bind(&request.area_uuid)
            .bind(&request.name)
            .bind(&request.icon_name)
            .bind(&request.folder_color)
            .bind(&request.parent_folder_uuid)
            .bind(sort_order)
            .bind(if auto_sync_to_vector_db { 1 } else { 0 })
            .bind(if vcs_export_allowed { 1 } else { 0 })
            .bind(if includes_private_data { 1 } else { 0 })
            .bind(serde_json::json!({}))
            .execute(p)
            .await?;
        }
    }

    // Emit folder created event
    if let Some(disp) = dispatcher {
        let folder = load_folder_by_uuid(pool, &folder_uuid).await.ok();
        let event = Event::new(
            "module_docs_folder_created",
            EventPayload::new(json!({
                "entity_type": "folder",
                "entity_id": folder_uuid,
                "organization_uuid": organization_uuid,
                "data": folder.as_ref().map(|f| json!({
                    "name": f.name,
                    "icon_name": f.icon_name,
                    "folder_color": f.folder_color,
                    "area_uuid": f.area_uuid,
                    "parent_folder_uuid": f.parent_folder_uuid,
                    "sort_order": f.sort_order
                })).unwrap_or(json!({}))
            }))
        )
        .with_organization(organization_uuid)
        .with_user(user_uuid);

        disp.emit(event).await;
    }

    Ok(folder_uuid)
}

/// Delete a folder from the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `folder_uuid` - UUID of the folder to delete
/// * `organization_uuid` - UUID of the organization
/// * `user_uuid` - UUID of the user deleting the folder
///
/// # Returns
/// Returns `()` on success
///
/// # Errors
/// Returns `DocsFolderDatabaseError` if:
/// - User does not belong to the organization
/// - User does not have permission to delete folders
/// - Folder does not belong to the organization
/// - Folder contains sub-folders or pages
/// - Folder not found
/// - Database operation fails
pub async fn delete_folder(
    pool: &DatabasePool,
    folder_uuid: &str,
    organization_uuid: &str,
    user_uuid: &str,
    dispatcher: Option<&EventDispatcher>,
) -> Result<(), DocsFolderDatabaseError> {
    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(pool, user_uuid, organization_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            DocsFolderDatabaseError::Database(e.into())
        })?;

    if !belongs {
        return Err(DocsFolderDatabaseError::UserNotInOrganization);
    }

    // Load folder to verify it belongs to the organization
    let folder = load_folder_by_uuid(pool, folder_uuid).await?;

    if folder.organization_uuid != organization_uuid {
        return Err(DocsFolderDatabaseError::FolderNotInOrganization);
    }

    // Check area member permissions
    let member_perms = load_area_member_permissions(pool, &folder.area_uuid, user_uuid)
        .await
        .map_err(|e| match e {
            DocsAreaDatabaseError::Database(e) => DocsFolderDatabaseError::Database(e),
            DocsAreaDatabaseError::Sql(e) => DocsFolderDatabaseError::Sql(e),
            DocsAreaDatabaseError::UserNotInOrganization => {
                DocsFolderDatabaseError::UserNotInOrganization
            }
            DocsAreaDatabaseError::PermissionDenied => DocsFolderDatabaseError::PermissionDenied,
            DocsAreaDatabaseError::AreaNotFound => DocsFolderDatabaseError::AreaNotFound,
            DocsAreaDatabaseError::AreaNotInOrganization => {
                DocsFolderDatabaseError::AreaNotInOrganization
            }
            DocsAreaDatabaseError::EmptyShortName => {
                DocsFolderDatabaseError::Database(
                    flextide_core::database::DatabaseError::PoolCreationFailed(
                        sqlx::Error::RowNotFound,
                    ),
                )
            }
        })?;

    // Check if user has permission to delete folders
    let can_delete = if let Some(perms) = &member_perms {
        // Check if user is a member and has can_delete_folders permission
        perms.admin || perms.role == "owner" || perms.can_delete_folders
    } else {
        // If not a member, user cannot delete folders
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
                DocsFolderDatabaseError::Database(db_err)
            }
            flextide_core::user::UserDatabaseError::Sql(sql_err) => {
                DocsFolderDatabaseError::Sql(sql_err)
            }
            _ => DocsFolderDatabaseError::Database(
                flextide_core::database::DatabaseError::PoolCreationFailed(
                    sqlx::Error::RowNotFound,
                ),
            ),
        }
    })?;

    if !can_delete && !has_super_admin {
        return Err(DocsFolderDatabaseError::PermissionDenied);
    }

    // Check if folder has sub-folders
    let sub_folders_count: i64 = match pool {
        DatabasePool::MySql(p) => {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM module_docs_folders WHERE parent_folder_uuid = ?",
            )
            .bind(folder_uuid)
            .fetch_one(p)
            .await?
        }
        DatabasePool::Postgres(p) => {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM module_docs_folders WHERE parent_folder_uuid = $1",
            )
            .bind(folder_uuid)
            .fetch_one(p)
            .await?
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM module_docs_folders WHERE parent_folder_uuid = ?1",
            )
            .bind(folder_uuid)
            .fetch_one(p)
            .await?
        }
    };

    if sub_folders_count > 0 {
        return Err(DocsFolderDatabaseError::FolderNotEmpty);
    }

    // Check if folder has pages
    let pages_count: i64 = match pool {
        DatabasePool::MySql(p) => {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM module_docs_pages WHERE folder_uuid = ?",
            )
            .bind(folder_uuid)
            .fetch_one(p)
            .await?
        }
        DatabasePool::Postgres(p) => {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM module_docs_pages WHERE folder_uuid = $1",
            )
            .bind(folder_uuid)
            .fetch_one(p)
            .await?
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM module_docs_pages WHERE folder_uuid = ?1",
            )
            .bind(folder_uuid)
            .fetch_one(p)
            .await?
        }
    };

    if pages_count > 0 {
        return Err(DocsFolderDatabaseError::FolderNotEmpty);
    }

    // Delete folder
    match pool {
        DatabasePool::MySql(p) => {
            let result = sqlx::query("DELETE FROM module_docs_folders WHERE uuid = ?")
                .bind(folder_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsFolderDatabaseError::FolderNotFound);
            }
        }
        DatabasePool::Postgres(p) => {
            let result = sqlx::query("DELETE FROM module_docs_folders WHERE uuid = $1")
                .bind(folder_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsFolderDatabaseError::FolderNotFound);
            }
        }
        DatabasePool::Sqlite(p) => {
            let result = sqlx::query("DELETE FROM module_docs_folders WHERE uuid = ?1")
                .bind(folder_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsFolderDatabaseError::FolderNotFound);
            }
        }
    }

    // Emit folder deleted event (before deletion, we already have the folder data)
    if let Some(disp) = dispatcher {
        let event = Event::new(
            "module_docs_folder_deleted",
            EventPayload::new(json!({
                "entity_type": "folder",
                "entity_id": folder_uuid,
                "organization_uuid": organization_uuid,
                "data": json!({
                    "name": folder.name,
                    "icon_name": folder.icon_name,
                    "folder_color": folder.folder_color,
                    "area_uuid": folder.area_uuid,
                    "parent_folder_uuid": folder.parent_folder_uuid,
                    "sort_order": folder.sort_order
                })
            }))
        )
        .with_organization(organization_uuid)
        .with_user(user_uuid);

        disp.emit(event).await;
    }

    Ok(())
}

/// Update folder name
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `folder_uuid` - UUID of the folder to update
/// * `organization_uuid` - UUID of the organization
/// * `user_uuid` - UUID of the user updating the folder
/// * `name` - New name for the folder
///
/// # Returns
/// Returns `()` on success
///
/// # Errors
/// Returns `DocsFolderDatabaseError` if:
/// - User does not belong to the organization
/// - User does not have permission to edit folders
/// - Folder does not belong to the organization
/// - Name is empty
/// - Folder not found
/// - Database operation fails
pub async fn update_folder_name(
    pool: &DatabasePool,
    folder_uuid: &str,
    organization_uuid: &str,
    user_uuid: &str,
    name: String,
    dispatcher: Option<&EventDispatcher>,
) -> Result<(), DocsFolderDatabaseError> {
    // Validate name
    if name.trim().is_empty() {
        return Err(DocsFolderDatabaseError::EmptyName);
    }

    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(pool, user_uuid, organization_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            DocsFolderDatabaseError::Database(e.into())
        })?;

    if !belongs {
        return Err(DocsFolderDatabaseError::UserNotInOrganization);
    }

    // Load folder to verify it belongs to the organization
    let folder = load_folder_by_uuid(pool, folder_uuid).await?;

    if folder.organization_uuid != organization_uuid {
        return Err(DocsFolderDatabaseError::FolderNotInOrganization);
    }

    // Check area member permissions
    let member_perms = load_area_member_permissions(pool, &folder.area_uuid, user_uuid)
        .await
        .map_err(|e| match e {
            DocsAreaDatabaseError::Database(e) => DocsFolderDatabaseError::Database(e),
            DocsAreaDatabaseError::Sql(e) => DocsFolderDatabaseError::Sql(e),
            DocsAreaDatabaseError::UserNotInOrganization => {
                DocsFolderDatabaseError::UserNotInOrganization
            }
            DocsAreaDatabaseError::PermissionDenied => DocsFolderDatabaseError::PermissionDenied,
            DocsAreaDatabaseError::AreaNotFound => DocsFolderDatabaseError::AreaNotFound,
            DocsAreaDatabaseError::AreaNotInOrganization => {
                DocsFolderDatabaseError::AreaNotInOrganization
            }
            DocsAreaDatabaseError::EmptyShortName => {
                DocsFolderDatabaseError::Database(
                    flextide_core::database::DatabaseError::PoolCreationFailed(
                        sqlx::Error::RowNotFound,
                    ),
                )
            }
        })?;

    // Check if user has permission to edit folders
    let can_edit = if let Some(perms) = &member_perms {
        // Check if user is a member and has can_edit_folders permission
        perms.admin || perms.role == "owner" || perms.can_edit_folders
    } else {
        // If not a member, user cannot edit folders
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
                DocsFolderDatabaseError::Database(db_err)
            }
            flextide_core::user::UserDatabaseError::Sql(sql_err) => {
                DocsFolderDatabaseError::Sql(sql_err)
            }
            _ => DocsFolderDatabaseError::Database(
                flextide_core::database::DatabaseError::PoolCreationFailed(
                    sqlx::Error::RowNotFound,
                ),
            ),
        }
    })?;

    if !can_edit && !has_super_admin {
        return Err(DocsFolderDatabaseError::PermissionDenied);
    }

    // Update folder name
    match pool {
        DatabasePool::MySql(p) => {
            let result = sqlx::query("UPDATE module_docs_folders SET name = ? WHERE uuid = ?")
                .bind(&name)
                .bind(folder_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsFolderDatabaseError::FolderNotFound);
            }
        }
        DatabasePool::Postgres(p) => {
            let result = sqlx::query("UPDATE module_docs_folders SET name = $1 WHERE uuid = $2")
                .bind(&name)
                .bind(folder_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsFolderDatabaseError::FolderNotFound);
            }
        }
        DatabasePool::Sqlite(p) => {
            let result = sqlx::query("UPDATE module_docs_folders SET name = ?1 WHERE uuid = ?2")
                .bind(&name)
                .bind(folder_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsFolderDatabaseError::FolderNotFound);
            }
        }
    }

    // Emit folder updated event
    if let Some(disp) = dispatcher {
        let folder = load_folder_by_uuid(pool, folder_uuid).await.ok();
        let event = Event::new(
            "module_docs_folder_updated",
            EventPayload::new(json!({
                "entity_type": "folder",
                "entity_id": folder_uuid,
                "organization_uuid": organization_uuid,
                "data": folder.as_ref().map(|f| json!({
                    "name": f.name,
                    "icon_name": f.icon_name,
                    "folder_color": f.folder_color,
                    "area_uuid": f.area_uuid,
                    "parent_folder_uuid": f.parent_folder_uuid,
                    "sort_order": f.sort_order
                })).unwrap_or(json!({}))
            }))
        )
        .with_organization(organization_uuid)
        .with_user(user_uuid);

        disp.emit(event).await;
    }

    Ok(())
}

/// Update folder properties (auto_sync_to_vector_db, vcs_export_allowed, includes_private_data, metadata)
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `folder_uuid` - UUID of the folder to update
/// * `organization_uuid` - UUID of the organization
/// * `user_uuid` - UUID of the user updating the folder
/// * `auto_sync_to_vector_db` - Whether to auto-sync to vector DB
/// * `vcs_export_allowed` - Whether VCS export is allowed
/// * `includes_private_data` - Whether folder includes private data
/// * `metadata` - JSON metadata (must be a valid JSON object)
/// * `dispatcher` - Optional event dispatcher
///
/// # Returns
/// Returns `()` on success
///
/// # Errors
/// Returns `DocsFolderDatabaseError` if:
/// - User does not belong to the organization
/// - User does not have permission to edit folder properties
/// - Folder does not belong to the organization
/// - Metadata is not a valid JSON object
/// - Database operation fails
pub async fn update_folder_properties(
    pool: &DatabasePool,
    folder_uuid: &str,
    organization_uuid: &str,
    user_uuid: &str,
    auto_sync_to_vector_db: bool,
    vcs_export_allowed: bool,
    includes_private_data: bool,
    metadata: serde_json::Value,
    dispatcher: Option<&EventDispatcher>,
) -> Result<(), DocsFolderDatabaseError> {
    // Validate metadata is a JSON object (not array or primitive)
    if !metadata.is_object() {
        return Err(DocsFolderDatabaseError::InvalidMetadata);
    }

    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(pool, user_uuid, organization_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            DocsFolderDatabaseError::Database(e.into())
        })?;

    if !belongs {
        return Err(DocsFolderDatabaseError::UserNotInOrganization);
    }

    // Load folder to verify it belongs to the organization
    let folder = load_folder_by_uuid(pool, folder_uuid).await?;

    if folder.organization_uuid != organization_uuid {
        return Err(DocsFolderDatabaseError::FolderNotInOrganization);
    }

    // Check area member permissions
    let member_perms = load_area_member_permissions(pool, &folder.area_uuid, user_uuid)
        .await
        .map_err(|e| match e {
            DocsAreaDatabaseError::Database(e) => DocsFolderDatabaseError::Database(e),
            DocsAreaDatabaseError::Sql(e) => DocsFolderDatabaseError::Sql(e),
            DocsAreaDatabaseError::UserNotInOrganization => {
                DocsFolderDatabaseError::UserNotInOrganization
            }
            DocsAreaDatabaseError::PermissionDenied => DocsFolderDatabaseError::PermissionDenied,
            DocsAreaDatabaseError::AreaNotFound => DocsFolderDatabaseError::AreaNotFound,
            DocsAreaDatabaseError::AreaNotInOrganization => {
                DocsFolderDatabaseError::AreaNotInOrganization
            }
            DocsAreaDatabaseError::EmptyShortName => {
                DocsFolderDatabaseError::Database(
                    flextide_core::database::DatabaseError::PoolCreationFailed(
                        sqlx::Error::RowNotFound,
                    ),
                )
            }
        })?;

    // Check if user has permission to edit folder properties
    let can_edit_properties = if let Some(perms) = &member_perms {
        // Check if user is a member and has can_edit_folder_properties permission
        perms.admin || perms.role == "owner" || perms.can_edit_folder_properties
    } else {
        // If not a member, user cannot edit folder properties
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
                DocsFolderDatabaseError::Database(db_err)
            }
            flextide_core::user::UserDatabaseError::Sql(sql_err) => {
                DocsFolderDatabaseError::Sql(sql_err)
            }
            _ => DocsFolderDatabaseError::Database(
                flextide_core::database::DatabaseError::PoolCreationFailed(
                    sqlx::Error::RowNotFound,
                ),
            ),
        }
    })?;

    if !can_edit_properties && !has_super_admin {
        return Err(DocsFolderDatabaseError::PermissionDenied);
    }

    // Update folder properties
    match pool {
        DatabasePool::MySql(p) => {
            let result = sqlx::query(
                "UPDATE module_docs_folders SET auto_sync_to_vector_db = ?, vcs_export_allowed = ?, includes_private_data = ?, metadata = ? WHERE uuid = ?"
            )
                .bind(if auto_sync_to_vector_db { 1 } else { 0 })
                .bind(if vcs_export_allowed { 1 } else { 0 })
                .bind(if includes_private_data { 1 } else { 0 })
                .bind(&metadata)
                .bind(folder_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsFolderDatabaseError::FolderNotFound);
            }
        }
        DatabasePool::Postgres(p) => {
            let result = sqlx::query(
                "UPDATE module_docs_folders SET auto_sync_to_vector_db = $1, vcs_export_allowed = $2, includes_private_data = $3, metadata = $4 WHERE uuid = $5"
            )
                .bind(if auto_sync_to_vector_db { 1 } else { 0 })
                .bind(if vcs_export_allowed { 1 } else { 0 })
                .bind(if includes_private_data { 1 } else { 0 })
                .bind(&metadata)
                .bind(folder_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsFolderDatabaseError::FolderNotFound);
            }
        }
        DatabasePool::Sqlite(p) => {
            let result = sqlx::query(
                "UPDATE module_docs_folders SET auto_sync_to_vector_db = ?1, vcs_export_allowed = ?2, includes_private_data = ?3, metadata = ?4 WHERE uuid = ?5"
            )
                .bind(if auto_sync_to_vector_db { 1 } else { 0 })
                .bind(if vcs_export_allowed { 1 } else { 0 })
                .bind(if includes_private_data { 1 } else { 0 })
                .bind(&metadata)
                .bind(folder_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsFolderDatabaseError::FolderNotFound);
            }
        }
    }

    // Emit folder properties updated event
    if let Some(disp) = dispatcher {
        let folder = load_folder_by_uuid(pool, folder_uuid).await.ok();
        let event = Event::new(
            "module_docs_folder_properties_updated",
            EventPayload::new(json!({
                "entity_type": "folder",
                "entity_id": folder_uuid,
                "organization_uuid": organization_uuid,
                "data": folder.as_ref().map(|f| json!({
                    "auto_sync_to_vector_db": f.auto_sync_to_vector_db,
                    "vcs_export_allowed": f.vcs_export_allowed,
                    "includes_private_data": f.includes_private_data,
                    "metadata": f.metadata
                })).unwrap_or(json!({}))
            }))
        )
        .with_organization(organization_uuid)
        .with_user(user_uuid);

        disp.emit(event).await;
    }

    Ok(())
}

/// Update a folder in the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `folder_uuid` - UUID of the folder to update
/// * `organization_uuid` - UUID of the organization
/// * `user_uuid` - UUID of the user updating the folder
/// * `request` - Folder update request
///
/// # Returns
/// Returns `()` on success
///
/// # Errors
/// Returns `DocsFolderDatabaseError` if:
/// - User does not belong to the organization
/// - User does not have permission to edit folders
/// - Folder does not belong to the organization
/// - Database operation fails
pub async fn update_folder(
    pool: &DatabasePool,
    folder_uuid: &str,
    organization_uuid: &str,
    user_uuid: &str,
    request: UpdateDocsFolderRequest,
    dispatcher: Option<&EventDispatcher>,
) -> Result<(), DocsFolderDatabaseError> {
    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(pool, user_uuid, organization_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            DocsFolderDatabaseError::Database(e.into())
        })?;

    if !belongs {
        return Err(DocsFolderDatabaseError::UserNotInOrganization);
    }

    // Load folder to verify it belongs to the organization
    let folder = load_folder_by_uuid(pool, folder_uuid).await?;

    if folder.organization_uuid != organization_uuid {
        return Err(DocsFolderDatabaseError::FolderNotInOrganization);
    }

    // Check area member permissions
    let member_perms = load_area_member_permissions(pool, &folder.area_uuid, user_uuid)
        .await
        .map_err(|e| match e {
            DocsAreaDatabaseError::Database(e) => DocsFolderDatabaseError::Database(e),
            DocsAreaDatabaseError::Sql(e) => DocsFolderDatabaseError::Sql(e),
            DocsAreaDatabaseError::UserNotInOrganization => {
                DocsFolderDatabaseError::UserNotInOrganization
            }
            DocsAreaDatabaseError::PermissionDenied => DocsFolderDatabaseError::PermissionDenied,
            DocsAreaDatabaseError::AreaNotFound => DocsFolderDatabaseError::AreaNotFound,
            DocsAreaDatabaseError::AreaNotInOrganization => {
                DocsFolderDatabaseError::AreaNotInOrganization
            }
            DocsAreaDatabaseError::EmptyShortName => {
                DocsFolderDatabaseError::Database(
                    flextide_core::database::DatabaseError::PoolCreationFailed(
                        sqlx::Error::RowNotFound,
                    ),
                )
            }
        })?;

    // Check if user has permission to edit folders
    let can_edit = if let Some(perms) = &member_perms {
        // Check if user is a member and has can_edit_folders permission
        perms.admin || perms.role == "owner" || perms.can_edit_folders
    } else {
        // If not a member, user cannot edit folders
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
                DocsFolderDatabaseError::Database(db_err)
            }
            flextide_core::user::UserDatabaseError::Sql(sql_err) => {
                DocsFolderDatabaseError::Sql(sql_err)
            }
            _ => DocsFolderDatabaseError::Database(
                flextide_core::database::DatabaseError::PoolCreationFailed(
                    sqlx::Error::RowNotFound,
                ),
            ),
        }
    })?;

    if !can_edit && !has_super_admin {
        return Err(DocsFolderDatabaseError::PermissionDenied);
    }

    // Validate name if provided
    if let Some(ref name) = request.name {
        if name.trim().is_empty() {
            return Err(DocsFolderDatabaseError::EmptyName);
        }
        if name.len() > 255 {
            return Err(DocsFolderDatabaseError::Database(
                flextide_core::database::DatabaseError::PoolCreationFailed(
                    sqlx::Error::RowNotFound,
                ),
            ));
        }
    }

    // Update folder fields
    match pool {
        DatabasePool::MySql(p) => {
            if let Some(ref name) = request.name {
                if let Some(ref icon_name) = request.icon_name {
                    if let Some(ref folder_color) = request.folder_color {
                        sqlx::query("UPDATE module_docs_folders SET name = ?, icon_name = ?, folder_color = ? WHERE uuid = ?")
                            .bind(name)
                            .bind(icon_name)
                            .bind(folder_color)
                            .bind(folder_uuid)
                            .execute(p)
                            .await?;
                    } else if request.folder_color.is_some() {
                        sqlx::query("UPDATE module_docs_folders SET name = ?, icon_name = ?, folder_color = NULL WHERE uuid = ?")
                            .bind(name)
                            .bind(icon_name)
                            .bind(folder_uuid)
                            .execute(p)
                            .await?;
                    } else {
                        sqlx::query("UPDATE module_docs_folders SET name = ?, icon_name = ? WHERE uuid = ?")
                            .bind(name)
                            .bind(icon_name)
                            .bind(folder_uuid)
                            .execute(p)
                            .await?;
                    }
                } else if request.icon_name.is_some() {
                    if let Some(ref folder_color) = request.folder_color {
                        sqlx::query("UPDATE module_docs_folders SET name = ?, icon_name = NULL, folder_color = ? WHERE uuid = ?")
                            .bind(name)
                            .bind(folder_color)
                            .bind(folder_uuid)
                            .execute(p)
                            .await?;
                    } else if request.folder_color.is_some() {
                        sqlx::query("UPDATE module_docs_folders SET name = ?, icon_name = NULL, folder_color = NULL WHERE uuid = ?")
                            .bind(name)
                            .bind(folder_uuid)
                            .execute(p)
                            .await?;
                    } else {
                        sqlx::query("UPDATE module_docs_folders SET name = ?, icon_name = NULL WHERE uuid = ?")
                            .bind(name)
                            .bind(folder_uuid)
                            .execute(p)
                            .await?;
                    }
                } else if let Some(ref folder_color) = request.folder_color {
                    sqlx::query("UPDATE module_docs_folders SET name = ?, folder_color = ? WHERE uuid = ?")
                        .bind(name)
                        .bind(folder_color)
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                } else if request.folder_color.is_some() {
                    sqlx::query("UPDATE module_docs_folders SET name = ?, folder_color = NULL WHERE uuid = ?")
                        .bind(name)
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                } else {
                    sqlx::query("UPDATE module_docs_folders SET name = ? WHERE uuid = ?")
                        .bind(name)
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                }
            } else if let Some(ref icon_name) = request.icon_name {
                if let Some(ref folder_color) = request.folder_color {
                    sqlx::query("UPDATE module_docs_folders SET icon_name = ?, folder_color = ? WHERE uuid = ?")
                        .bind(icon_name)
                        .bind(folder_color)
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                } else if request.folder_color.is_some() {
                    sqlx::query("UPDATE module_docs_folders SET icon_name = ?, folder_color = NULL WHERE uuid = ?")
                        .bind(icon_name)
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                } else {
                    sqlx::query("UPDATE module_docs_folders SET icon_name = ? WHERE uuid = ?")
                        .bind(icon_name)
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                }
            } else if request.icon_name.is_some() {
                if let Some(ref folder_color) = request.folder_color {
                    sqlx::query("UPDATE module_docs_folders SET icon_name = NULL, folder_color = ? WHERE uuid = ?")
                        .bind(folder_color)
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                } else if request.folder_color.is_some() {
                    sqlx::query("UPDATE module_docs_folders SET icon_name = NULL, folder_color = NULL WHERE uuid = ?")
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                } else {
                    sqlx::query("UPDATE module_docs_folders SET icon_name = NULL WHERE uuid = ?")
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                }
            } else if let Some(ref folder_color) = request.folder_color {
                sqlx::query("UPDATE module_docs_folders SET folder_color = ? WHERE uuid = ?")
                    .bind(folder_color)
                    .bind(folder_uuid)
                    .execute(p)
                    .await?;
            } else if request.folder_color.is_some() {
                sqlx::query("UPDATE module_docs_folders SET folder_color = NULL WHERE uuid = ?")
                    .bind(folder_uuid)
                    .execute(p)
                    .await?;
            }
        }
        DatabasePool::Postgres(p) => {
            if let Some(ref name) = request.name {
                if let Some(ref icon_name) = request.icon_name {
                    if let Some(ref folder_color) = request.folder_color {
                        sqlx::query("UPDATE module_docs_folders SET name = $1, icon_name = $2, folder_color = $3 WHERE uuid = $4")
                            .bind(name)
                            .bind(icon_name)
                            .bind(folder_color)
                            .bind(folder_uuid)
                            .execute(p)
                            .await?;
                    } else if request.folder_color.is_some() {
                        sqlx::query("UPDATE module_docs_folders SET name = $1, icon_name = $2, folder_color = NULL WHERE uuid = $3")
                            .bind(name)
                            .bind(icon_name)
                            .bind(folder_uuid)
                            .execute(p)
                            .await?;
                    } else {
                        sqlx::query("UPDATE module_docs_folders SET name = $1, icon_name = $2 WHERE uuid = $3")
                            .bind(name)
                            .bind(icon_name)
                            .bind(folder_uuid)
                            .execute(p)
                            .await?;
                    }
                } else if request.icon_name.is_some() {
                    if let Some(ref folder_color) = request.folder_color {
                        sqlx::query("UPDATE module_docs_folders SET name = $1, icon_name = NULL, folder_color = $2 WHERE uuid = $3")
                            .bind(name)
                            .bind(folder_color)
                            .bind(folder_uuid)
                            .execute(p)
                            .await?;
                    } else if request.folder_color.is_some() {
                        sqlx::query("UPDATE module_docs_folders SET name = $1, icon_name = NULL, folder_color = NULL WHERE uuid = $2")
                            .bind(name)
                            .bind(folder_uuid)
                            .execute(p)
                            .await?;
                    } else {
                        sqlx::query("UPDATE module_docs_folders SET name = $1, icon_name = NULL WHERE uuid = $2")
                            .bind(name)
                            .bind(folder_uuid)
                            .execute(p)
                            .await?;
                    }
                } else if let Some(ref folder_color) = request.folder_color {
                    sqlx::query("UPDATE module_docs_folders SET name = $1, folder_color = $2 WHERE uuid = $3")
                        .bind(name)
                        .bind(folder_color)
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                } else if request.folder_color.is_some() {
                    sqlx::query("UPDATE module_docs_folders SET name = $1, folder_color = NULL WHERE uuid = $2")
                        .bind(name)
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                } else {
                    sqlx::query("UPDATE module_docs_folders SET name = $1 WHERE uuid = $2")
                        .bind(name)
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                }
            } else if let Some(ref icon_name) = request.icon_name {
                if let Some(ref folder_color) = request.folder_color {
                    sqlx::query("UPDATE module_docs_folders SET icon_name = $1, folder_color = $2 WHERE uuid = $3")
                        .bind(icon_name)
                        .bind(folder_color)
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                } else if request.folder_color.is_some() {
                    sqlx::query("UPDATE module_docs_folders SET icon_name = $1, folder_color = NULL WHERE uuid = $2")
                        .bind(icon_name)
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                } else {
                    sqlx::query("UPDATE module_docs_folders SET icon_name = $1 WHERE uuid = $2")
                        .bind(icon_name)
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                }
            } else if request.icon_name.is_some() {
                if let Some(ref folder_color) = request.folder_color {
                    sqlx::query("UPDATE module_docs_folders SET icon_name = NULL, folder_color = $1 WHERE uuid = $2")
                        .bind(folder_color)
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                } else if request.folder_color.is_some() {
                    sqlx::query("UPDATE module_docs_folders SET icon_name = NULL, folder_color = NULL WHERE uuid = $1")
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                } else {
                    sqlx::query("UPDATE module_docs_folders SET icon_name = NULL WHERE uuid = $1")
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                }
            } else if let Some(ref folder_color) = request.folder_color {
                sqlx::query("UPDATE module_docs_folders SET folder_color = $1 WHERE uuid = $2")
                    .bind(folder_color)
                    .bind(folder_uuid)
                    .execute(p)
                    .await?;
            } else if request.folder_color.is_some() {
                sqlx::query("UPDATE module_docs_folders SET folder_color = NULL WHERE uuid = $1")
                    .bind(folder_uuid)
                    .execute(p)
                    .await?;
            }
        }
        DatabasePool::Sqlite(p) => {
            if let Some(ref name) = request.name {
                if let Some(ref icon_name) = request.icon_name {
                    if let Some(ref folder_color) = request.folder_color {
                        sqlx::query("UPDATE module_docs_folders SET name = ?1, icon_name = ?2, folder_color = ?3 WHERE uuid = ?4")
                            .bind(name)
                            .bind(icon_name)
                            .bind(folder_color)
                            .bind(folder_uuid)
                            .execute(p)
                            .await?;
                    } else if request.folder_color.is_some() {
                        sqlx::query("UPDATE module_docs_folders SET name = ?1, icon_name = ?2, folder_color = NULL WHERE uuid = ?3")
                            .bind(name)
                            .bind(icon_name)
                            .bind(folder_uuid)
                            .execute(p)
                            .await?;
                    } else {
                        sqlx::query("UPDATE module_docs_folders SET name = ?1, icon_name = ?2 WHERE uuid = ?3")
                            .bind(name)
                            .bind(icon_name)
                            .bind(folder_uuid)
                            .execute(p)
                            .await?;
                    }
                } else if request.icon_name.is_some() {
                    if let Some(ref folder_color) = request.folder_color {
                        sqlx::query("UPDATE module_docs_folders SET name = ?1, icon_name = NULL, folder_color = ?2 WHERE uuid = ?3")
                            .bind(name)
                            .bind(folder_color)
                            .bind(folder_uuid)
                            .execute(p)
                            .await?;
                    } else if request.folder_color.is_some() {
                        sqlx::query("UPDATE module_docs_folders SET name = ?1, icon_name = NULL, folder_color = NULL WHERE uuid = ?2")
                            .bind(name)
                            .bind(folder_uuid)
                            .execute(p)
                            .await?;
                    } else {
                        sqlx::query("UPDATE module_docs_folders SET name = ?1, icon_name = NULL WHERE uuid = ?2")
                            .bind(name)
                            .bind(folder_uuid)
                            .execute(p)
                            .await?;
                    }
                } else if let Some(ref folder_color) = request.folder_color {
                    sqlx::query("UPDATE module_docs_folders SET name = ?1, folder_color = ?2 WHERE uuid = ?3")
                        .bind(name)
                        .bind(folder_color)
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                } else if request.folder_color.is_some() {
                    sqlx::query("UPDATE module_docs_folders SET name = ?1, folder_color = NULL WHERE uuid = ?2")
                        .bind(name)
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                } else {
                    sqlx::query("UPDATE module_docs_folders SET name = ?1 WHERE uuid = ?2")
                        .bind(name)
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                }
            } else if let Some(ref icon_name) = request.icon_name {
                if let Some(ref folder_color) = request.folder_color {
                    sqlx::query("UPDATE module_docs_folders SET icon_name = ?1, folder_color = ?2 WHERE uuid = ?3")
                        .bind(icon_name)
                        .bind(folder_color)
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                } else if request.folder_color.is_some() {
                    sqlx::query("UPDATE module_docs_folders SET icon_name = ?1, folder_color = NULL WHERE uuid = ?2")
                        .bind(icon_name)
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                } else {
                    sqlx::query("UPDATE module_docs_folders SET icon_name = ?1 WHERE uuid = ?2")
                        .bind(icon_name)
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                }
            } else if request.icon_name.is_some() {
                if let Some(ref folder_color) = request.folder_color {
                    sqlx::query("UPDATE module_docs_folders SET icon_name = NULL, folder_color = ?1 WHERE uuid = ?2")
                        .bind(folder_color)
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                } else if request.folder_color.is_some() {
                    sqlx::query("UPDATE module_docs_folders SET icon_name = NULL, folder_color = NULL WHERE uuid = ?1")
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                } else {
                    sqlx::query("UPDATE module_docs_folders SET icon_name = NULL WHERE uuid = ?1")
                        .bind(folder_uuid)
                        .execute(p)
                        .await?;
                }
            } else if let Some(ref folder_color) = request.folder_color {
                sqlx::query("UPDATE module_docs_folders SET folder_color = ?1 WHERE uuid = ?2")
                    .bind(folder_color)
                    .bind(folder_uuid)
                    .execute(p)
                    .await?;
            } else if request.folder_color.is_some() {
                sqlx::query("UPDATE module_docs_folders SET folder_color = NULL WHERE uuid = ?1")
                    .bind(folder_uuid)
                    .execute(p)
                    .await?;
            }
        }
    }

    // Emit folder updated event
    if let Some(disp) = dispatcher {
        let folder = load_folder_by_uuid(pool, folder_uuid).await.ok();
        let event = Event::new(
            "module_docs_folder_updated",
            EventPayload::new(json!({
                "entity_type": "folder",
                "entity_id": folder_uuid,
                "organization_uuid": organization_uuid,
                "data": folder.as_ref().map(|f| json!({
                    "name": f.name,
                    "icon_name": f.icon_name,
                    "folder_color": f.folder_color,
                    "area_uuid": f.area_uuid,
                    "parent_folder_uuid": f.parent_folder_uuid,
                    "sort_order": f.sort_order
                })).unwrap_or(json!({}))
            }))
        )
        .with_organization(organization_uuid)
        .with_user(user_uuid);

        disp.emit(event).await;
    }

    Ok(())
}

/// Reorder a folder (change sort_order)
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `folder_uuid` - UUID of the folder to reorder
/// * `organization_uuid` - UUID of the organization
/// * `user_uuid` - UUID of the user reordering the folder
/// * `sort_order` - New sort order value
///
/// # Returns
/// Returns `()` on success
///
/// # Errors
/// Returns `DocsFolderDatabaseError` if:
/// - User does not belong to the organization
/// - User does not have permission to edit folders
/// - Folder does not belong to the organization
/// - Folder not found
/// - Database operation fails
pub async fn reorder_folder(
    pool: &DatabasePool,
    folder_uuid: &str,
    organization_uuid: &str,
    user_uuid: &str,
    sort_order: i32,
    dispatcher: Option<&EventDispatcher>,
) -> Result<(), DocsFolderDatabaseError> {
    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(pool, user_uuid, organization_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            DocsFolderDatabaseError::Database(e.into())
        })?;

    if !belongs {
        return Err(DocsFolderDatabaseError::UserNotInOrganization);
    }

    // Load folder to verify it belongs to the organization
    let folder = load_folder_by_uuid(pool, folder_uuid).await?;

    if folder.organization_uuid != organization_uuid {
        return Err(DocsFolderDatabaseError::FolderNotInOrganization);
    }

    // Load area to verify it exists (permissions are checked via member_perms)
    let _area = load_area_by_uuid(pool, &folder.area_uuid)
        .await
        .map_err(|e| match e {
            DocsAreaDatabaseError::AreaNotFound => DocsFolderDatabaseError::AreaNotFound,
            DocsAreaDatabaseError::Database(e) => DocsFolderDatabaseError::Database(e),
            DocsAreaDatabaseError::Sql(e) => DocsFolderDatabaseError::Sql(e),
            DocsAreaDatabaseError::UserNotInOrganization => {
                DocsFolderDatabaseError::UserNotInOrganization
            }
            DocsAreaDatabaseError::PermissionDenied => DocsFolderDatabaseError::PermissionDenied,
            DocsAreaDatabaseError::AreaNotInOrganization => {
                DocsFolderDatabaseError::AreaNotInOrganization
            }
            DocsAreaDatabaseError::EmptyShortName => {
                DocsFolderDatabaseError::Database(
                    flextide_core::database::DatabaseError::PoolCreationFailed(
                        sqlx::Error::RowNotFound,
                    ),
                )
            }
        })?;

    // Check area member permissions
    let member_perms = load_area_member_permissions(pool, &folder.area_uuid, user_uuid)
        .await
        .map_err(|e| match e {
            DocsAreaDatabaseError::Database(e) => DocsFolderDatabaseError::Database(e),
            DocsAreaDatabaseError::Sql(e) => DocsFolderDatabaseError::Sql(e),
            DocsAreaDatabaseError::UserNotInOrganization => {
                DocsFolderDatabaseError::UserNotInOrganization
            }
            DocsAreaDatabaseError::PermissionDenied => DocsFolderDatabaseError::PermissionDenied,
            DocsAreaDatabaseError::AreaNotFound => DocsFolderDatabaseError::AreaNotFound,
            DocsAreaDatabaseError::AreaNotInOrganization => {
                DocsFolderDatabaseError::AreaNotInOrganization
            }
            DocsAreaDatabaseError::EmptyShortName => {
                DocsFolderDatabaseError::Database(
                    flextide_core::database::DatabaseError::PoolCreationFailed(
                        sqlx::Error::RowNotFound,
                    ),
                )
            }
        })?;

    // Check if user has permission to edit folders
    // User must be a member of the area with can_edit_folders permission, or be admin/owner
    let can_edit = if let Some(perms) = &member_perms {
        perms.admin || perms.role == "owner" || perms.can_edit_folders
    } else {
        // If not a member, user cannot edit folders (even if area is public)
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
                DocsFolderDatabaseError::Database(db_err)
            }
            flextide_core::user::UserDatabaseError::Sql(sql_err) => {
                DocsFolderDatabaseError::Sql(sql_err)
            }
            _ => DocsFolderDatabaseError::Database(
                flextide_core::database::DatabaseError::PoolCreationFailed(
                    sqlx::Error::RowNotFound,
                ),
            ),
        }
    })?;

    if !can_edit && !has_super_admin {
        return Err(DocsFolderDatabaseError::PermissionDenied);
    }

    // Update sort_order
    match pool {
        DatabasePool::MySql(p) => {
            let result = sqlx::query("UPDATE module_docs_folders SET sort_order = ? WHERE uuid = ?")
                .bind(sort_order)
                .bind(folder_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsFolderDatabaseError::FolderNotFound);
            }
        }
        DatabasePool::Postgres(p) => {
            let result = sqlx::query("UPDATE module_docs_folders SET sort_order = $1 WHERE uuid = $2")
                .bind(sort_order)
                .bind(folder_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsFolderDatabaseError::FolderNotFound);
            }
        }
        DatabasePool::Sqlite(p) => {
            let result = sqlx::query("UPDATE module_docs_folders SET sort_order = ?1 WHERE uuid = ?2")
                .bind(sort_order)
                .bind(folder_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsFolderDatabaseError::FolderNotFound);
            }
        }
    }

    // Emit folder updated event (reorder is also an update)
    if let Some(disp) = dispatcher {
        let folder = load_folder_by_uuid(pool, folder_uuid).await.ok();
        let event = Event::new(
            "module_docs_folder_updated",
            EventPayload::new(json!({
                "entity_type": "folder",
                "entity_id": folder_uuid,
                "organization_uuid": organization_uuid,
                "data": folder.as_ref().map(|f| json!({
                    "name": f.name,
                    "icon_name": f.icon_name,
                    "folder_color": f.folder_color,
                    "area_uuid": f.area_uuid,
                    "parent_folder_uuid": f.parent_folder_uuid,
                    "sort_order": f.sort_order
                })).unwrap_or(json!({}))
            }))
        )
        .with_organization(organization_uuid)
        .with_user(user_uuid);

        disp.emit(event).await;
    }

    Ok(())
}

/// Move a folder to a different parent and/or position
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `folder_uuid` - UUID of the folder to move
/// * `organization_uuid` - UUID of the organization
/// * `user_uuid` - UUID of the user moving the folder
/// * `parent_folder_uuid` - New parent folder UUID (None for root)
/// * `sort_order` - New sort order value
///
/// # Returns
/// Returns `()` on success
///
/// # Errors
/// Returns `DocsFolderDatabaseError` if:
/// - User does not belong to the organization
/// - User does not have permission to edit folders
/// - Folder does not belong to the organization
/// - Parent folder does not belong to the organization or area
/// - Folder would be moved into itself or a descendant
/// - Folder not found
/// - Database operation fails
pub async fn move_folder(
    pool: &DatabasePool,
    folder_uuid: &str,
    organization_uuid: &str,
    user_uuid: &str,
    parent_folder_uuid: Option<String>,
    sort_order: i32,
    dispatcher: Option<&EventDispatcher>,
) -> Result<(), DocsFolderDatabaseError> {
    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(pool, user_uuid, organization_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            DocsFolderDatabaseError::Database(e.into())
        })?;

    if !belongs {
        return Err(DocsFolderDatabaseError::UserNotInOrganization);
    }

    // Load folder to verify it belongs to the organization
    let folder = load_folder_by_uuid(pool, folder_uuid).await?;

    if folder.organization_uuid != organization_uuid {
        return Err(DocsFolderDatabaseError::FolderNotInOrganization);
    }

    // Validate parent folder if provided
    if let Some(ref parent_uuid) = parent_folder_uuid {
        // Prevent moving folder into itself
        if parent_uuid == folder_uuid {
            return Err(DocsFolderDatabaseError::Database(
                flextide_core::database::DatabaseError::PoolCreationFailed(
                    sqlx::Error::RowNotFound,
                ),
            ));
        }

        let parent_folder = load_folder_by_uuid(pool, parent_uuid).await?;
        
        if parent_folder.organization_uuid != organization_uuid {
            return Err(DocsFolderDatabaseError::FolderNotInOrganization);
        }
        
        if parent_folder.area_uuid != folder.area_uuid {
            return Err(DocsFolderDatabaseError::AreaNotInOrganization);
        }

        // Prevent moving folder into a descendant (would create a cycle)
        let mut current_parent_uuid = parent_folder.parent_folder_uuid.clone();
        while let Some(parent_uuid_str) = current_parent_uuid {
            if parent_uuid_str == folder_uuid {
                return Err(DocsFolderDatabaseError::Database(
                    flextide_core::database::DatabaseError::PoolCreationFailed(
                        sqlx::Error::RowNotFound,
                    ),
                ));
            }
            let parent = load_folder_by_uuid(pool, &parent_uuid_str).await.ok();
            current_parent_uuid = parent.and_then(|p| p.parent_folder_uuid.clone());
        }
    }

    // Load area to check permissions (we need it to check if area is public)
    let _area = load_area_by_uuid(pool, &folder.area_uuid)
        .await
        .map_err(|e| match e {
            DocsAreaDatabaseError::Database(e) => DocsFolderDatabaseError::Database(e),
            DocsAreaDatabaseError::Sql(e) => DocsFolderDatabaseError::Sql(e),
            DocsAreaDatabaseError::UserNotInOrganization => {
                DocsFolderDatabaseError::UserNotInOrganization
            }
            DocsAreaDatabaseError::PermissionDenied => DocsFolderDatabaseError::PermissionDenied,
            DocsAreaDatabaseError::AreaNotFound => DocsFolderDatabaseError::AreaNotFound,
            DocsAreaDatabaseError::AreaNotInOrganization => {
                DocsFolderDatabaseError::AreaNotInOrganization
            }
            DocsAreaDatabaseError::EmptyShortName => {
                DocsFolderDatabaseError::Database(
                    flextide_core::database::DatabaseError::PoolCreationFailed(
                        sqlx::Error::RowNotFound,
                    ),
                )
            }
        })?;

    // Check area member permissions
    let member_perms = load_area_member_permissions(pool, &folder.area_uuid, user_uuid)
        .await
        .map_err(|e| match e {
            DocsAreaDatabaseError::Database(e) => DocsFolderDatabaseError::Database(e),
            DocsAreaDatabaseError::Sql(e) => DocsFolderDatabaseError::Sql(e),
            DocsAreaDatabaseError::UserNotInOrganization => {
                DocsFolderDatabaseError::UserNotInOrganization
            }
            DocsAreaDatabaseError::PermissionDenied => DocsFolderDatabaseError::PermissionDenied,
            DocsAreaDatabaseError::AreaNotFound => DocsFolderDatabaseError::AreaNotFound,
            DocsAreaDatabaseError::AreaNotInOrganization => {
                DocsFolderDatabaseError::AreaNotInOrganization
            }
            DocsAreaDatabaseError::EmptyShortName => {
                DocsFolderDatabaseError::Database(
                    flextide_core::database::DatabaseError::PoolCreationFailed(
                        sqlx::Error::RowNotFound,
                    ),
                )
            }
        })?;

    // Check if user has permission to edit folders
    let can_edit = if let Some(perms) = &member_perms {
        perms.admin || perms.role == "owner" || perms.can_edit_folders
    } else {
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
                DocsFolderDatabaseError::Database(db_err)
            }
            flextide_core::user::UserDatabaseError::Sql(sql_err) => {
                DocsFolderDatabaseError::Sql(sql_err)
            }
            _ => DocsFolderDatabaseError::Database(
                flextide_core::database::DatabaseError::PoolCreationFailed(
                    sqlx::Error::RowNotFound,
                ),
            ),
        }
    })?;

    if !can_edit && !has_super_admin {
        return Err(DocsFolderDatabaseError::PermissionDenied);
    }

    // Update parent_folder_uuid and sort_order
    match pool {
        DatabasePool::MySql(p) => {
            let result = sqlx::query(
                "UPDATE module_docs_folders SET parent_folder_uuid = ?, sort_order = ? WHERE uuid = ?"
            )
                .bind(&parent_folder_uuid)
                .bind(sort_order)
                .bind(folder_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsFolderDatabaseError::FolderNotFound);
            }
        }
        DatabasePool::Postgres(p) => {
            let result = sqlx::query(
                "UPDATE module_docs_folders SET parent_folder_uuid = $1, sort_order = $2 WHERE uuid = $3"
            )
                .bind(&parent_folder_uuid)
                .bind(sort_order)
                .bind(folder_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsFolderDatabaseError::FolderNotFound);
            }
        }
        DatabasePool::Sqlite(p) => {
            let result = sqlx::query(
                "UPDATE module_docs_folders SET parent_folder_uuid = ?1, sort_order = ?2 WHERE uuid = ?3"
            )
                .bind(&parent_folder_uuid)
                .bind(sort_order)
                .bind(folder_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsFolderDatabaseError::FolderNotFound);
            }
        }
    }

    // Emit folder updated event
    if let Some(disp) = dispatcher {
        let folder = load_folder_by_uuid(pool, folder_uuid).await.ok();
        let event = Event::new(
            "module_docs_folder_updated",
            EventPayload::new(json!({
                "entity_type": "folder",
                "entity_id": folder_uuid,
                "organization_uuid": organization_uuid,
                "data": folder.as_ref().map(|f| json!({
                    "name": f.name,
                    "icon_name": f.icon_name,
                    "folder_color": f.folder_color,
                    "area_uuid": f.area_uuid,
                    "parent_folder_uuid": f.parent_folder_uuid,
                    "sort_order": f.sort_order
                })).unwrap_or(json!({}))
            }))
        )
        .with_organization(organization_uuid)
        .with_user(user_uuid);

        disp.emit(event).await;
    }

    Ok(())
}

