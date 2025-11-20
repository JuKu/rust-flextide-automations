//! Docs Area module
//!
//! Provides functionality for managing documentation areas, including database operations and permission checks.

use chrono::{DateTime, Utc};
use flextide_core::database::{DatabaseError, DatabasePool};
use flextide_core::events::{Event, EventDispatcher, EventPayload};
use flextide_core::user::{user_belongs_to_organization, user_has_permission};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use thiserror::Error;

/// Error type for Docs area database operations
#[derive(Debug, Error)]
pub enum DocsAreaDatabaseError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("SQL execution error: {0}")]
    Sql(#[from] sqlx::Error),

    #[error("User does not belong to this organization")]
    UserNotInOrganization,

    #[error("User does not have permission to perform this action")]
    PermissionDenied,

    #[error("Area not found")]
    AreaNotFound,

    #[error("Area does not belong to this organization")]
    AreaNotInOrganization,

    #[error("Short name cannot be empty")]
    EmptyShortName,
}

/// Docs Area data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsArea {
    pub uuid: String,
    pub organization_uuid: String,
    pub short_name: String,
    pub description: Option<String>,
    pub icon_name: Option<String>,
    pub public: bool,
    pub visible: bool,
    pub deletable: bool,
    pub creator_uuid: String,
    pub created_at: DateTime<Utc>,
}

/// Request structure for creating a new area
#[derive(Debug, Deserialize)]
pub struct CreateDocsAreaRequest {
    pub short_name: String,
    pub description: Option<String>,
    pub icon_name: Option<String>,
    pub public: Option<bool>,
    pub visible: Option<bool>,
    pub deletable: Option<bool>,
}

/// Request structure for updating an area
#[derive(Debug, Deserialize)]
pub struct UpdateDocsAreaRequest {
    pub short_name: Option<String>,
    pub description: Option<String>,
    pub icon_name: Option<String>,
    pub public: Option<bool>,
    pub visible: Option<bool>,
    pub deletable: Option<bool>,
}

/// Area member permissions structure
#[derive(Debug, Clone)]
pub struct AreaMemberPermissions {
    pub role: String,
    pub can_view: bool,
    pub can_add_pages: bool,
    pub can_edit_pages: bool,
    pub can_edit_own_pages: bool,
    pub can_archive_pages: bool,
    pub can_archive_own_pages: bool,
    pub can_delete_pages: bool,
    pub can_delete_own_pages: bool,
    pub can_export_pages: bool,
    pub admin: bool,
}

/// Load area member permissions for a user in an area
pub async fn load_area_member_permissions(
    pool: &DatabasePool,
    area_uuid: &str,
    user_uuid: &str,
) -> Result<Option<AreaMemberPermissions>, DocsAreaDatabaseError> {
    match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query(
                "SELECT role, can_view, can_add_pages, can_edit_pages, can_edit_own_pages,
                 can_archive_pages, can_archive_own_pages, can_delete_pages, can_delete_own_pages,
                 can_export_pages, admin
                 FROM module_docs_area_members
                 WHERE area_uuid = ? AND user_uuid = ?",
            )
            .bind(area_uuid)
            .bind(user_uuid)
            .fetch_optional(p)
            .await?;

            match row {
                Some(row) => Ok(Some(AreaMemberPermissions {
                    role: row.get("role"),
                    can_view: row.get::<i64, _>("can_view") != 0,
                    can_add_pages: row.get::<i64, _>("can_add_pages") != 0,
                    can_edit_pages: row.get::<i64, _>("can_edit_pages") != 0,
                    can_edit_own_pages: row.get::<i64, _>("can_edit_own_pages") != 0,
                    can_archive_pages: row.get::<i64, _>("can_archive_pages") != 0,
                    can_archive_own_pages: row.get::<i64, _>("can_archive_own_pages") != 0,
                    can_delete_pages: row.get::<i64, _>("can_delete_pages") != 0,
                    can_delete_own_pages: row.get::<i64, _>("can_delete_own_pages") != 0,
                    can_export_pages: row.get::<i64, _>("can_export_pages") != 0,
                    admin: row.get::<i64, _>("admin") != 0,
                })),
                None => Ok(None),
            }
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT role, can_view, can_add_pages, can_edit_pages, can_edit_own_pages,
                 can_archive_pages, can_archive_own_pages, can_delete_pages, can_delete_own_pages,
                 can_export_pages, admin
                 FROM module_docs_area_members
                 WHERE area_uuid = $1 AND user_uuid = $2",
            )
            .bind(area_uuid)
            .bind(user_uuid)
            .fetch_optional(p)
            .await?;

            match row {
                Some(row) => Ok(Some(AreaMemberPermissions {
                    role: row.get("role"),
                    can_view: row.get::<i32, _>("can_view") != 0,
                    can_add_pages: row.get::<i32, _>("can_add_pages") != 0,
                    can_edit_pages: row.get::<i32, _>("can_edit_pages") != 0,
                    can_edit_own_pages: row.get::<i32, _>("can_edit_own_pages") != 0,
                    can_archive_pages: row.get::<i32, _>("can_archive_pages") != 0,
                    can_archive_own_pages: row.get::<i32, _>("can_archive_own_pages") != 0,
                    can_delete_pages: row.get::<i32, _>("can_delete_pages") != 0,
                    can_delete_own_pages: row.get::<i32, _>("can_delete_own_pages") != 0,
                    can_export_pages: row.get::<i32, _>("can_export_pages") != 0,
                    admin: row.get::<i32, _>("admin") != 0,
                })),
                None => Ok(None),
            }
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT role, can_view, can_add_pages, can_edit_pages, can_edit_own_pages,
                 can_archive_pages, can_archive_own_pages, can_delete_pages, can_delete_own_pages,
                 can_export_pages, admin
                 FROM module_docs_area_members
                 WHERE area_uuid = ?1 AND user_uuid = ?2",
            )
            .bind(area_uuid)
            .bind(user_uuid)
            .fetch_optional(p)
            .await?;

            match row {
                Some(row) => Ok(Some(AreaMemberPermissions {
                    role: row.get("role"),
                    can_view: row.get::<i64, _>("can_view") != 0,
                    can_add_pages: row.get::<i64, _>("can_add_pages") != 0,
                    can_edit_pages: row.get::<i64, _>("can_edit_pages") != 0,
                    can_edit_own_pages: row.get::<i64, _>("can_edit_own_pages") != 0,
                    can_archive_pages: row.get::<i64, _>("can_archive_pages") != 0,
                    can_archive_own_pages: row.get::<i64, _>("can_archive_own_pages") != 0,
                    can_delete_pages: row.get::<i64, _>("can_delete_pages") != 0,
                    can_delete_own_pages: row.get::<i64, _>("can_delete_own_pages") != 0,
                    can_export_pages: row.get::<i64, _>("can_export_pages") != 0,
                    admin: row.get::<i64, _>("admin") != 0,
                })),
                None => Ok(None),
            }
        }
    }
}

/// Load an area from the database by UUID
pub async fn load_area_by_uuid(
    pool: &DatabasePool,
    area_uuid: &str,
) -> Result<DocsArea, DocsAreaDatabaseError> {
    match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query(
                "SELECT uuid, organization_uuid, short_name, description, icon_name,
                 public, visible, deletable, creator_uuid, created_at
                 FROM module_docs_areas WHERE uuid = ?",
            )
            .bind(area_uuid)
            .fetch_optional(p)
            .await?;

            match row {
                Some(row) => Ok(DocsArea {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    short_name: row.get("short_name"),
                    description: row.get("description"),
                    icon_name: row.get("icon_name"),
                    public: row.get::<i64, _>("public") != 0,
                    visible: row.get::<i64, _>("visible") != 0,
                    deletable: row.get::<i64, _>("deletable") != 0,
                    creator_uuid: row.get("creator_uuid"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                }),
                None => Err(DocsAreaDatabaseError::AreaNotFound),
            }
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT uuid, organization_uuid, short_name, description, icon_name,
                 public, visible, deletable, creator_uuid, created_at
                 FROM module_docs_areas WHERE uuid = $1",
            )
            .bind(area_uuid)
            .fetch_optional(p)
            .await?;

            match row {
                Some(row) => Ok(DocsArea {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    short_name: row.get("short_name"),
                    description: row.get("description"),
                    icon_name: row.get("icon_name"),
                    public: row.get::<i32, _>("public") != 0,
                    visible: row.get::<i32, _>("visible") != 0,
                    deletable: row.get::<i32, _>("deletable") != 0,
                    creator_uuid: row.get("creator_uuid"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                }),
                None => Err(DocsAreaDatabaseError::AreaNotFound),
            }
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT uuid, organization_uuid, short_name, description, icon_name,
                 public, visible, deletable, creator_uuid, created_at
                 FROM module_docs_areas WHERE uuid = ?1",
            )
            .bind(area_uuid)
            .fetch_optional(p)
            .await?;

            match row {
                Some(row) => Ok(DocsArea {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    short_name: row.get("short_name"),
                    description: row.get("description"),
                    icon_name: row.get("icon_name"),
                    public: row.get::<i64, _>("public") != 0,
                    visible: row.get::<i64, _>("visible") != 0,
                    deletable: row.get::<i64, _>("deletable") != 0,
                    creator_uuid: row.get("creator_uuid"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                }),
                None => Err(DocsAreaDatabaseError::AreaNotFound),
            }
        }
    }
}

/// Create a new area in the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `organization_uuid` - UUID of the organization the area belongs to
/// * `user_uuid` - UUID of the user creating the area
/// * `request` - Area creation request
///
/// # Returns
/// Returns the UUID of the newly created area
///
/// # Errors
/// Returns `DocsAreaDatabaseError` if:
/// - User does not belong to the organization
/// - User does not have permission to create areas
/// - Short name is empty
/// - Database operation fails
pub async fn create_area(
    pool: &DatabasePool,
    organization_uuid: &str,
    user_uuid: &str,
    request: CreateDocsAreaRequest,
    dispatcher: Option<&EventDispatcher>,
) -> Result<String, DocsAreaDatabaseError> {
    // Validate short name
    if request.short_name.trim().is_empty() {
        return Err(DocsAreaDatabaseError::EmptyShortName);
    }

    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(pool, user_uuid, organization_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            DocsAreaDatabaseError::Database(e.into())
        })?;

    if !belongs {
        return Err(DocsAreaDatabaseError::UserNotInOrganization);
    }

    // Check permission
    let has_permission = user_has_permission(
        pool,
        user_uuid,
        organization_uuid,
        "module_docs_can_create_areas",
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error checking permission: {}", e);
        DocsAreaDatabaseError::Database(e.into())
    })?;

    if !has_permission {
        return Err(DocsAreaDatabaseError::PermissionDenied);
    }

    // Create area
    let area_uuid = uuid::Uuid::new_v4().to_string();
    let public = if request.public.unwrap_or(false) { 1 } else { 0 };
    let visible = if request.visible.unwrap_or(true) { 1 } else { 0 };
    let deletable = if request.deletable.unwrap_or(true) { 1 } else { 0 };

    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query(
                "INSERT INTO module_docs_areas
                 (uuid, organization_uuid, short_name, description, icon_name,
                  public, visible, deletable, creator_uuid, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&area_uuid)
            .bind(organization_uuid)
            .bind(&request.short_name)
            .bind(&request.description)
            .bind(&request.icon_name)
            .bind(public)
            .bind(visible)
            .bind(deletable)
            .bind(user_uuid)
            .bind(Utc::now())
            .execute(p)
            .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO module_docs_areas
                 (uuid, organization_uuid, short_name, description, icon_name,
                  public, visible, deletable, creator_uuid, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
            )
            .bind(&area_uuid)
            .bind(organization_uuid)
            .bind(&request.short_name)
            .bind(&request.description)
            .bind(&request.icon_name)
            .bind(public)
            .bind(visible)
            .bind(deletable)
            .bind(user_uuid)
            .bind(Utc::now())
            .execute(p)
            .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO module_docs_areas
                 (uuid, organization_uuid, short_name, description, icon_name,
                  public, visible, deletable, creator_uuid, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            )
            .bind(&area_uuid)
            .bind(organization_uuid)
            .bind(&request.short_name)
            .bind(&request.description)
            .bind(&request.icon_name)
            .bind(public)
            .bind(visible)
            .bind(deletable)
            .bind(user_uuid)
            .bind(Utc::now())
            .execute(p)
            .await?;
        }
    }

    // Emit area created event
    if let Some(disp) = dispatcher {
        let area = load_area_by_uuid(pool, &area_uuid).await.ok();
        let event = Event::new(
            "module_docs_area_created",
            EventPayload::new(json!({
                "entity_type": "area",
                "entity_id": area_uuid,
                "organization_uuid": organization_uuid,
                "data": area.as_ref().map(|a| json!({
                    "short_name": a.short_name,
                    "description": a.description,
                    "icon_name": a.icon_name,
                    "public": a.public,
                    "visible": a.visible,
                    "deletable": a.deletable
                })).unwrap_or(json!({}))
            }))
        )
        .with_organization(organization_uuid)
        .with_user(user_uuid);

        disp.emit(event).await;
    }

    Ok(area_uuid)
}

/// Update an area in the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `area_uuid` - UUID of the area to update
/// * `organization_uuid` - UUID of the organization (for verification)
/// * `user_uuid` - UUID of the user updating the area
/// * `request` - Update request with fields to update
///
/// # Errors
/// Returns `DocsAreaDatabaseError` if:
/// - User does not belong to the organization
/// - Area does not belong to the organization
/// - User does not have permission to edit the area
/// - Database operation fails
pub async fn update_area(
    pool: &DatabasePool,
    area_uuid: &str,
    organization_uuid: &str,
    user_uuid: &str,
    request: UpdateDocsAreaRequest,
    dispatcher: Option<&EventDispatcher>,
) -> Result<(), DocsAreaDatabaseError> {
    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(pool, user_uuid, organization_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            DocsAreaDatabaseError::Database(e.into())
        })?;

    if !belongs {
        return Err(DocsAreaDatabaseError::UserNotInOrganization);
    }

    // Load area to verify it belongs to the organization
    let area = load_area_by_uuid(pool, area_uuid).await?;

    if area.organization_uuid != organization_uuid {
        return Err(DocsAreaDatabaseError::AreaNotInOrganization);
    }

    // Check permission: can_edit_all_areas or (can_edit_own_areas and user is creator)
    let has_edit_all = user_has_permission(
        pool,
        user_uuid,
        organization_uuid,
        "module_docs_can_edit_all_areas",
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error checking permission: {}", e);
        DocsAreaDatabaseError::Database(e.into())
    })?;

    let has_edit_own = user_has_permission(
        pool,
        user_uuid,
        organization_uuid,
        "module_docs_can_edit_own_areas",
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error checking permission: {}", e);
        DocsAreaDatabaseError::Database(e.into())
    })?;

    let is_creator = area.creator_uuid == user_uuid;

    if !has_edit_all && !(has_edit_own && is_creator) {
        // Also check area member permissions
        let member_perms = load_area_member_permissions(pool, area_uuid, user_uuid).await?;
        if let Some(perms) = member_perms {
            if perms.admin || perms.role == "owner" {
                // Allow if user is admin or owner in the area
            } else {
                return Err(DocsAreaDatabaseError::PermissionDenied);
            }
        } else {
            return Err(DocsAreaDatabaseError::PermissionDenied);
        }
    }

    // Validate short name if provided
    if let Some(ref short_name) = request.short_name {
        if short_name.trim().is_empty() {
            return Err(DocsAreaDatabaseError::EmptyShortName);
        }
    }

    // Build dynamic UPDATE query
    let mut update_fields = Vec::new();

    if request.short_name.is_some() {
        update_fields.push("short_name = ?");
    }
    if request.description.is_some() {
        update_fields.push("description = ?");
    }
    if request.icon_name.is_some() {
        update_fields.push("icon_name = ?");
    }
    if request.public.is_some() {
        update_fields.push("public = ?");
    }
    if request.visible.is_some() {
        update_fields.push("visible = ?");
    }
    if request.deletable.is_some() {
        update_fields.push("deletable = ?");
    }

    if update_fields.is_empty() {
        return Ok(()); // Nothing to update
    }

    let update_clause = update_fields.join(", ");

    match pool {
        DatabasePool::MySql(p) => {
            let query_str = format!(
                "UPDATE module_docs_areas SET {} WHERE uuid = ?",
                update_clause
            );
            let mut query = sqlx::query(&query_str);

            if let Some(ref v) = request.short_name {
                query = query.bind(v);
            }
            if let Some(ref v) = request.description {
                query = query.bind(v);
            }
            if let Some(ref v) = request.icon_name {
                query = query.bind(v);
            }
            if let Some(v) = request.public {
                query = query.bind(if v { 1 } else { 0 });
            }
            if let Some(v) = request.visible {
                query = query.bind(if v { 1 } else { 0 });
            }
            if let Some(v) = request.deletable {
                query = query.bind(if v { 1 } else { 0 });
            }
            query = query.bind(area_uuid);

            let result = query.execute(p).await?;
            if result.rows_affected() == 0 {
                return Err(DocsAreaDatabaseError::AreaNotFound);
            }
        }
        DatabasePool::Postgres(p) => {
            let mut bind_index = 1;
            let mut update_fields_pg = Vec::new();

            if request.short_name.is_some() {
                update_fields_pg.push(format!("short_name = ${}", bind_index));
                bind_index += 1;
            }
            if request.description.is_some() {
                update_fields_pg.push(format!("description = ${}", bind_index));
                bind_index += 1;
            }
            if request.icon_name.is_some() {
                update_fields_pg.push(format!("icon_name = ${}", bind_index));
                bind_index += 1;
            }
            if request.public.is_some() {
                update_fields_pg.push(format!("public = ${}", bind_index));
                bind_index += 1;
            }
            if request.visible.is_some() {
                update_fields_pg.push(format!("visible = ${}", bind_index));
                bind_index += 1;
            }
            if request.deletable.is_some() {
                update_fields_pg.push(format!("deletable = ${}", bind_index));
                bind_index += 1;
            }

            let update_clause_pg = update_fields_pg.join(", ");
            let query_str = format!(
                "UPDATE module_docs_areas SET {} WHERE uuid = ${}",
                update_clause_pg, bind_index
            );
            let mut query = sqlx::query(&query_str);

            if let Some(ref v) = request.short_name {
                query = query.bind(v);
            }
            if let Some(ref v) = request.description {
                query = query.bind(v);
            }
            if let Some(ref v) = request.icon_name {
                query = query.bind(v);
            }
            if let Some(v) = request.public {
                query = query.bind(if v { 1 } else { 0 });
            }
            if let Some(v) = request.visible {
                query = query.bind(if v { 1 } else { 0 });
            }
            if let Some(v) = request.deletable {
                query = query.bind(if v { 1 } else { 0 });
            }
            query = query.bind(area_uuid);

            let result = query.execute(p).await?;
            if result.rows_affected() == 0 {
                return Err(DocsAreaDatabaseError::AreaNotFound);
            }
        }
        DatabasePool::Sqlite(p) => {
            let mut bind_index = 1;
            let mut update_fields_sqlite = Vec::new();

            if request.short_name.is_some() {
                update_fields_sqlite.push(format!("short_name = ?{}", bind_index));
                bind_index += 1;
            }
            if request.description.is_some() {
                update_fields_sqlite.push(format!("description = ?{}", bind_index));
                bind_index += 1;
            }
            if request.icon_name.is_some() {
                update_fields_sqlite.push(format!("icon_name = ?{}", bind_index));
                bind_index += 1;
            }
            if request.public.is_some() {
                update_fields_sqlite.push(format!("public = ?{}", bind_index));
                bind_index += 1;
            }
            if request.visible.is_some() {
                update_fields_sqlite.push(format!("visible = ?{}", bind_index));
                bind_index += 1;
            }
            if request.deletable.is_some() {
                update_fields_sqlite.push(format!("deletable = ?{}", bind_index));
                bind_index += 1;
            }

            let update_clause_sqlite = update_fields_sqlite.join(", ");
            let query_str = format!(
                "UPDATE module_docs_areas SET {} WHERE uuid = ?{}",
                update_clause_sqlite, bind_index
            );
            let mut query = sqlx::query(&query_str);

            if let Some(ref v) = request.short_name {
                query = query.bind(v);
            }
            if let Some(ref v) = request.description {
                query = query.bind(v);
            }
            if let Some(ref v) = request.icon_name {
                query = query.bind(v);
            }
            if let Some(v) = request.public {
                query = query.bind(if v { 1 } else { 0 });
            }
            if let Some(v) = request.visible {
                query = query.bind(if v { 1 } else { 0 });
            }
            if let Some(v) = request.deletable {
                query = query.bind(if v { 1 } else { 0 });
            }
            query = query.bind(area_uuid);

            let result = query.execute(p).await?;
            if result.rows_affected() == 0 {
                return Err(DocsAreaDatabaseError::AreaNotFound);
            }
        }
    }

    // Emit area updated event
    if let Some(disp) = dispatcher {
        let area = load_area_by_uuid(pool, area_uuid).await.ok();
        let event = Event::new(
            "module_docs_area_updated",
            EventPayload::new(json!({
                "entity_type": "area",
                "entity_id": area_uuid,
                "organization_uuid": organization_uuid,
                "data": area.as_ref().map(|a| json!({
                    "short_name": a.short_name,
                    "description": a.description,
                    "icon_name": a.icon_name,
                    "public": a.public,
                    "visible": a.visible,
                    "deletable": a.deletable
                })).unwrap_or(json!({}))
            }))
        )
        .with_organization(organization_uuid)
        .with_user(user_uuid);

        disp.emit(event).await;
    }

    Ok(())
}

/// Delete an area from the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `area_uuid` - UUID of the area to delete
/// * `organization_uuid` - UUID of the organization (for verification)
/// * `user_uuid` - UUID of the user deleting the area
///
/// # Errors
/// Returns `DocsAreaDatabaseError` if:
/// - User does not belong to the organization
/// - Area does not belong to the organization
/// - User does not have permission to delete the area
/// - Area is not deletable
/// - Database operation fails
pub async fn delete_area(
    pool: &DatabasePool,
    area_uuid: &str,
    organization_uuid: &str,
    user_uuid: &str,
    dispatcher: Option<&EventDispatcher>,
) -> Result<(), DocsAreaDatabaseError> {
    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(pool, user_uuid, organization_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            DocsAreaDatabaseError::Database(e.into())
        })?;

    if !belongs {
        return Err(DocsAreaDatabaseError::UserNotInOrganization);
    }

    // Load area to verify it belongs to the organization and check deletable flag
    // Also load it before deletion for event payload
    let area = load_area_by_uuid(pool, area_uuid).await?;

    if area.organization_uuid != organization_uuid {
        return Err(DocsAreaDatabaseError::AreaNotInOrganization);
    }

    if !area.deletable {
        return Err(DocsAreaDatabaseError::PermissionDenied);
    }

    // Check permission: can_delete_areas or (can_delete_own_areas and user is creator)
    let has_delete_all = user_has_permission(
        pool,
        user_uuid,
        organization_uuid,
        "module_docs_can_delete_areas",
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error checking permission: {}", e);
        DocsAreaDatabaseError::Database(e.into())
    })?;

    let has_delete_own = user_has_permission(
        pool,
        user_uuid,
        organization_uuid,
        "module_docs_can_delete_own_areas",
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error checking permission: {}", e);
        DocsAreaDatabaseError::Database(e.into())
    })?;

    let is_creator = area.creator_uuid == user_uuid;

    if !has_delete_all && !(has_delete_own && is_creator) {
        // Also check area member permissions
        let member_perms = load_area_member_permissions(pool, area_uuid, user_uuid).await?;
        if let Some(perms) = member_perms {
            if perms.admin || perms.role == "owner" {
                // Allow if user is admin or owner in the area
            } else {
                return Err(DocsAreaDatabaseError::PermissionDenied);
            }
        } else {
            return Err(DocsAreaDatabaseError::PermissionDenied);
        }
    }

    // Delete area (cascade will delete area_members)
    match pool {
        DatabasePool::MySql(p) => {
            let result = sqlx::query("DELETE FROM module_docs_areas WHERE uuid = ?")
                .bind(area_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsAreaDatabaseError::AreaNotFound);
            }
        }
        DatabasePool::Postgres(p) => {
            let result = sqlx::query("DELETE FROM module_docs_areas WHERE uuid = $1")
                .bind(area_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsAreaDatabaseError::AreaNotFound);
            }
        }
        DatabasePool::Sqlite(p) => {
            let result = sqlx::query("DELETE FROM module_docs_areas WHERE uuid = ?1")
                .bind(area_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(DocsAreaDatabaseError::AreaNotFound);
            }
        }
    }

    // Emit area deleted event (before deletion, we already have the area data)
    if let Some(disp) = dispatcher {
        let event = Event::new(
            "module_docs_area_deleted",
            EventPayload::new(json!({
                "entity_type": "area",
                "entity_id": area_uuid,
                "organization_uuid": organization_uuid,
                "data": json!({
                    "short_name": area.short_name,
                    "description": area.description,
                    "icon_name": area.icon_name,
                    "public": area.public,
                    "visible": area.visible,
                    "deletable": area.deletable
                })
            }))
        )
        .with_organization(organization_uuid)
        .with_user(user_uuid);

        disp.emit(event).await;
    }

    Ok(())
}

/// Area with membership information for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsAreaWithMembership {
    #[serde(flatten)]
    pub area: DocsArea,
    /// Whether the user is a direct member of this area
    pub is_member: bool,
}

/// List all areas accessible to a user in an organization
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `organization_uuid` - UUID of the organization
/// * `user_uuid` - UUID of the user
///
/// # Returns
/// Returns a list of areas the user can access. If the user has super_admin permission,
/// returns all areas in the organization. Otherwise, returns only areas where:
/// - The area is public, OR
/// - The user is a member of the area
///
/// Each area includes an `is_member` flag indicating if the user is a direct member.
///
/// # Errors
/// Returns `DocsAreaDatabaseError` if database operation fails
pub async fn list_accessible_areas(
    pool: &DatabasePool,
    organization_uuid: &str,
    user_uuid: &str,
) -> Result<Vec<DocsAreaWithMembership>, DocsAreaDatabaseError> {
    // Check if user has super_admin permission
    let has_super_admin = user_has_permission(
        pool,
        user_uuid,
        organization_uuid,
        "super_admin",
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error checking super_admin permission: {}", e);
        DocsAreaDatabaseError::Database(e.into())
    })?;

    let areas = if has_super_admin {
        // Super admin can see all areas in the organization
        match pool {
            DatabasePool::MySql(p) => {
                let rows = sqlx::query(
                    "SELECT uuid, organization_uuid, short_name, description, icon_name,
                     public, visible, deletable, creator_uuid, created_at
                     FROM module_docs_areas
                     WHERE organization_uuid = ? AND visible = 1
                     ORDER BY created_at DESC",
                )
                .bind(organization_uuid)
                .fetch_all(p)
                .await?;

                rows.into_iter()
                    .map(|row| {
                        let area_uuid: String = row.get("uuid");
                        (
                            DocsArea {
                                uuid: area_uuid.clone(),
                                organization_uuid: row.get("organization_uuid"),
                                short_name: row.get("short_name"),
                                description: row.get("description"),
                                icon_name: row.get("icon_name"),
                                public: row.get::<i64, _>("public") != 0,
                                visible: row.get::<i64, _>("visible") != 0,
                                deletable: row.get::<i64, _>("deletable") != 0,
                                creator_uuid: row.get("creator_uuid"),
                                created_at: row.get::<DateTime<Utc>, _>("created_at"),
                            },
                            area_uuid,
                        )
                    })
                    .collect::<Vec<_>>()
            }
            DatabasePool::Postgres(p) => {
                let rows = sqlx::query(
                    "SELECT uuid, organization_uuid, short_name, description, icon_name,
                     public, visible, deletable, creator_uuid, created_at
                     FROM module_docs_areas
                     WHERE organization_uuid = $1 AND visible = 1
                     ORDER BY created_at DESC",
                )
                .bind(organization_uuid)
                .fetch_all(p)
                .await?;

                rows.into_iter()
                    .map(|row| {
                        let area_uuid: String = row.get("uuid");
                        (
                            DocsArea {
                                uuid: area_uuid.clone(),
                                organization_uuid: row.get("organization_uuid"),
                                short_name: row.get("short_name"),
                                description: row.get("description"),
                                icon_name: row.get("icon_name"),
                                public: row.get::<i32, _>("public") != 0,
                                visible: row.get::<i32, _>("visible") != 0,
                                deletable: row.get::<i32, _>("deletable") != 0,
                                creator_uuid: row.get("creator_uuid"),
                                created_at: row.get::<DateTime<Utc>, _>("created_at"),
                            },
                            area_uuid,
                        )
                    })
                    .collect::<Vec<_>>()
            }
            DatabasePool::Sqlite(p) => {
                let rows = sqlx::query(
                    "SELECT uuid, organization_uuid, short_name, description, icon_name,
                     public, visible, deletable, creator_uuid, created_at
                     FROM module_docs_areas
                     WHERE organization_uuid = ?1 AND visible = 1
                     ORDER BY created_at DESC",
                )
                .bind(organization_uuid)
                .fetch_all(p)
                .await?;

                rows.into_iter()
                    .map(|row| {
                        let area_uuid: String = row.get("uuid");
                        (
                            DocsArea {
                                uuid: area_uuid.clone(),
                                organization_uuid: row.get("organization_uuid"),
                                short_name: row.get("short_name"),
                                description: row.get("description"),
                                icon_name: row.get("icon_name"),
                                public: row.get::<i64, _>("public") != 0,
                                visible: row.get::<i64, _>("visible") != 0,
                                deletable: row.get::<i64, _>("deletable") != 0,
                                creator_uuid: row.get("creator_uuid"),
                                created_at: row.get::<DateTime<Utc>, _>("created_at"),
                            },
                            area_uuid,
                        )
                    })
                    .collect::<Vec<_>>()
            }
        }
    } else {
        // Regular user: only areas they're a member of or public areas
        match pool {
            DatabasePool::MySql(p) => {
                let rows = sqlx::query(
                    "SELECT DISTINCT a.uuid, a.organization_uuid, a.short_name, a.description, a.icon_name,
                     a.public, a.visible, a.deletable, a.creator_uuid, a.created_at
                     FROM module_docs_areas a
                     LEFT JOIN module_docs_area_members m ON a.uuid = m.area_uuid AND m.user_uuid = ?
                     WHERE a.organization_uuid = ? AND a.visible = 1
                     AND (a.public = 1 OR m.user_uuid IS NOT NULL)
                     ORDER BY a.created_at DESC",
                )
                .bind(user_uuid)
                .bind(organization_uuid)
                .fetch_all(p)
                .await?;

                rows.into_iter()
                    .map(|row| {
                        let area_uuid: String = row.get("uuid");
                        (
                            DocsArea {
                                uuid: area_uuid.clone(),
                                organization_uuid: row.get("organization_uuid"),
                                short_name: row.get("short_name"),
                                description: row.get("description"),
                                icon_name: row.get("icon_name"),
                                public: row.get::<i64, _>("public") != 0,
                                visible: row.get::<i64, _>("visible") != 0,
                                deletable: row.get::<i64, _>("deletable") != 0,
                                creator_uuid: row.get("creator_uuid"),
                                created_at: row.get::<DateTime<Utc>, _>("created_at"),
                            },
                            area_uuid,
                        )
                    })
                    .collect::<Vec<_>>()
            }
            DatabasePool::Postgres(p) => {
                let rows = sqlx::query(
                    "SELECT DISTINCT a.uuid, a.organization_uuid, a.short_name, a.description, a.icon_name,
                     a.public, a.visible, a.deletable, a.creator_uuid, a.created_at
                     FROM module_docs_areas a
                     LEFT JOIN module_docs_area_members m ON a.uuid = m.area_uuid AND m.user_uuid = $1
                     WHERE a.organization_uuid = $2 AND a.visible = 1
                     AND (a.public = 1 OR m.user_uuid IS NOT NULL)
                     ORDER BY a.created_at DESC",
                )
                .bind(user_uuid)
                .bind(organization_uuid)
                .fetch_all(p)
                .await?;

                rows.into_iter()
                    .map(|row| {
                        let area_uuid: String = row.get("uuid");
                        (
                            DocsArea {
                                uuid: area_uuid.clone(),
                                organization_uuid: row.get("organization_uuid"),
                                short_name: row.get("short_name"),
                                description: row.get("description"),
                                icon_name: row.get("icon_name"),
                                public: row.get::<i32, _>("public") != 0,
                                visible: row.get::<i32, _>("visible") != 0,
                                deletable: row.get::<i32, _>("deletable") != 0,
                                creator_uuid: row.get("creator_uuid"),
                                created_at: row.get::<DateTime<Utc>, _>("created_at"),
                            },
                            area_uuid,
                        )
                    })
                    .collect::<Vec<_>>()
            }
            DatabasePool::Sqlite(p) => {
                let rows = sqlx::query(
                    "SELECT DISTINCT a.uuid, a.organization_uuid, a.short_name, a.description, a.icon_name,
                     a.public, a.visible, a.deletable, a.creator_uuid, a.created_at
                     FROM module_docs_areas a
                     LEFT JOIN module_docs_area_members m ON a.uuid = m.area_uuid AND m.user_uuid = ?1
                     WHERE a.organization_uuid = ?2 AND a.visible = 1
                     AND (a.public = 1 OR m.user_uuid IS NOT NULL)
                     ORDER BY a.created_at DESC",
                )
                .bind(user_uuid)
                .bind(organization_uuid)
                .fetch_all(p)
                .await?;

                rows.into_iter()
                    .map(|row| {
                        let area_uuid: String = row.get("uuid");
                        (
                            DocsArea {
                                uuid: area_uuid.clone(),
                                organization_uuid: row.get("organization_uuid"),
                                short_name: row.get("short_name"),
                                description: row.get("description"),
                                icon_name: row.get("icon_name"),
                                public: row.get::<i64, _>("public") != 0,
                                visible: row.get::<i64, _>("visible") != 0,
                                deletable: row.get::<i64, _>("deletable") != 0,
                                creator_uuid: row.get("creator_uuid"),
                                created_at: row.get::<DateTime<Utc>, _>("created_at"),
                            },
                            area_uuid,
                        )
                    })
                    .collect::<Vec<_>>()
            }
        }
    };

    // Check membership for each area
    let mut result = Vec::new();
    for (area, area_uuid) in areas {
        let is_member = match pool {
            DatabasePool::MySql(p) => {
                let row = sqlx::query(
                    "SELECT COUNT(*) as count FROM module_docs_area_members
                     WHERE area_uuid = ? AND user_uuid = ?",
                )
                .bind(&area_uuid)
                .bind(user_uuid)
                .fetch_one(p)
                .await?;
                let count: i64 = row.get("count");
                count > 0
            }
            DatabasePool::Postgres(p) => {
                let row = sqlx::query(
                    "SELECT COUNT(*) as count FROM module_docs_area_members
                     WHERE area_uuid = $1 AND user_uuid = $2",
                )
                .bind(&area_uuid)
                .bind(user_uuid)
                .fetch_one(p)
                .await?;
                let count: i64 = row.get("count");
                count > 0
            }
            DatabasePool::Sqlite(p) => {
                let row = sqlx::query(
                    "SELECT COUNT(*) as count FROM module_docs_area_members
                     WHERE area_uuid = ?1 AND user_uuid = ?2",
                )
                .bind(&area_uuid)
                .bind(user_uuid)
                .fetch_one(p)
                .await?;
                let count: i64 = row.get("count");
                count > 0
            }
        };

        result.push(DocsAreaWithMembership { area, is_member });
    }

    Ok(result)
}

