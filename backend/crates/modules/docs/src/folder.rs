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
                     sort_order, visible, created_at, activated
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
                     sort_order, visible, created_at, activated
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
                })
                .collect())
        }
        DatabasePool::Postgres(p) => {
            let folders = if let Some(parent_uuid) = parent_folder_uuid {
                sqlx::query(
                    "SELECT uuid, organization_uuid, area_uuid, name, icon_name, folder_color, parent_folder_uuid,
                     sort_order, visible, created_at, activated
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
                     sort_order, visible, created_at, activated
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
                })
                .collect())
        }
        DatabasePool::Sqlite(p) => {
            let folders = if let Some(parent_uuid) = parent_folder_uuid {
                sqlx::query(
                    "SELECT uuid, organization_uuid, area_uuid, name, icon_name, folder_color, parent_folder_uuid,
                     sort_order, visible, created_at, activated
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
                     sort_order, visible, created_at, activated
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
                })
                .collect())
        }
    }
}

/// Load a folder by UUID
async fn load_folder_by_uuid(
    pool: &DatabasePool,
    folder_uuid: &str,
) -> Result<DocsFolder, DocsFolderDatabaseError> {
    match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query(
                "SELECT uuid, organization_uuid, area_uuid, name, icon_name, folder_color, parent_folder_uuid,
                 sort_order, visible, created_at, activated
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
                }),
                None => Err(DocsFolderDatabaseError::FolderNotFound),
            }
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT uuid, organization_uuid, area_uuid, name, icon_name, folder_color, parent_folder_uuid,
                 sort_order, visible, created_at, activated
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
                }),
                None => Err(DocsFolderDatabaseError::FolderNotFound),
            }
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT uuid, organization_uuid, area_uuid, name, icon_name, folder_color, parent_folder_uuid,
                 sort_order, visible, created_at, activated
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

    // Check if user has permission to add pages (folders are similar to pages in terms of permissions)
    let can_add = if let Some(perms) = &member_perms {
        perms.admin || perms.role == "owner" || perms.can_add_pages
    } else {
        // If not a member, check if area is public and user has organization-level permission
        if area.public {
            user_has_permission(
                pool,
                user_uuid,
                organization_uuid,
                "module_docs_can_create_areas",
            )
            .await
            .map_err(|e| {
                tracing::error!("Database error checking permission: {}", e);
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
            })?
        } else {
            false
        }
    };

    if !can_add {
        return Err(DocsFolderDatabaseError::PermissionDenied);
    }

    // Validate parent folder if provided
    if let Some(ref parent_uuid) = request.parent_folder_uuid {
        let parent_folder = load_folder_by_uuid(pool, parent_uuid).await?;
        if parent_folder.organization_uuid != organization_uuid {
            return Err(DocsFolderDatabaseError::FolderNotInOrganization);
        }
        if parent_folder.area_uuid != request.area_uuid {
            return Err(DocsFolderDatabaseError::AreaNotInOrganization);
        }
    }

    // Create folder
    let folder_uuid = uuid::Uuid::new_v4().to_string();
    let sort_order = request.sort_order.unwrap_or(0);

    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query(
                "INSERT INTO module_docs_folders (uuid, organization_uuid, area_uuid, name, icon_name, folder_color, parent_folder_uuid, sort_order)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&folder_uuid)
            .bind(organization_uuid)
            .bind(&request.area_uuid)
            .bind(&request.name)
            .bind(&request.icon_name)
            .bind(&request.folder_color)
            .bind(&request.parent_folder_uuid)
            .bind(sort_order)
            .execute(p)
            .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO module_docs_folders (uuid, organization_uuid, area_uuid, name, icon_name, folder_color, parent_folder_uuid, sort_order)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            )
            .bind(&folder_uuid)
            .bind(organization_uuid)
            .bind(&request.area_uuid)
            .bind(&request.name)
            .bind(&request.icon_name)
            .bind(&request.folder_color)
            .bind(&request.parent_folder_uuid)
            .bind(sort_order)
            .execute(p)
            .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO module_docs_folders (uuid, organization_uuid, area_uuid, name, icon_name, folder_color, parent_folder_uuid, sort_order)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )
            .bind(&folder_uuid)
            .bind(organization_uuid)
            .bind(&request.area_uuid)
            .bind(&request.name)
            .bind(&request.icon_name)
            .bind(&request.folder_color)
            .bind(&request.parent_folder_uuid)
            .bind(sort_order)
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

    // Load area to check permissions
    let area = load_area_by_uuid(pool, &folder.area_uuid)
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

    // Check if user has permission to delete pages (folders use similar permissions)
    let can_delete = if let Some(perms) = &member_perms {
        perms.admin || perms.role == "owner" || perms.can_delete_pages
    } else {
        // If not a member, check if area is public and user has organization-level permission
        if area.public {
            user_has_permission(
                pool,
                user_uuid,
                organization_uuid,
                "module_docs_can_delete_areas",
            )
            .await
            .map_err(|e| {
                tracing::error!("Database error checking permission: {}", e);
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
            })?
        } else {
            false
        }
    };

    if !can_delete {
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

    // Load area to check permissions
    let area = load_area_by_uuid(pool, &folder.area_uuid)
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

    // Check if user has permission to edit pages (folders use similar permissions)
    let can_edit = if let Some(perms) = &member_perms {
        perms.admin || perms.role == "owner" || perms.can_edit_pages
    } else {
        // If not a member, check if area is public and user has organization-level permission
        if area.public {
            user_has_permission(
                pool,
                user_uuid,
                organization_uuid,
                "module_docs_can_edit_all_areas",
            )
            .await
            .map_err(|e| {
                tracing::error!("Database error checking permission: {}", e);
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
            })?
        } else {
            false
        }
    };

    if !can_edit {
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

    // Load area to check permissions
    let area = load_area_by_uuid(pool, &folder.area_uuid)
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

    // Check if user has permission to edit pages (folders use similar permissions)
    let can_edit = if let Some(perms) = &member_perms {
        perms.admin || perms.role == "owner" || perms.can_edit_pages
    } else {
        // If not a member, check if area is public and user has organization-level permission
        if area.public {
            user_has_permission(
                pool,
                user_uuid,
                organization_uuid,
                "module_docs_can_edit_all_areas",
            )
            .await
            .map_err(|e| {
                tracing::error!("Database error checking permission: {}", e);
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
            })?
        } else {
            false
        }
    };

    if !can_edit {
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

