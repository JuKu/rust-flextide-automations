//! Database operations for permission management

use crate::database::{DatabaseError, DatabasePool};
use crate::permissions::{
    CreatePermissionGroupRequest, CreatePermissionRequest, Permission, PermissionGroup, UserPermission,
};
use sqlx::Row;

/// Error type for permission database operations
#[derive(Debug, thiserror::Error)]
pub enum PermissionDatabaseError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("SQL execution error: {0}")]
    Sql(#[from] sqlx::Error),

    #[error("Permission group not found: {0}")]
    PermissionGroupNotFound(String),

    #[error("Permission not found: {0}")]
    PermissionNotFound(String),
}

/// Create a new permission group
pub async fn create_permission_group(
    pool: &DatabasePool,
    request: CreatePermissionGroupRequest,
) -> Result<PermissionGroup, PermissionDatabaseError> {
    let visible = request.visible.unwrap_or(true) as i32;
    let sort_order = request.sort_order.unwrap_or(0);

    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query(
                "INSERT INTO permission_groups (name, title, description, visible, sort_order)
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(&request.name)
            .bind(&request.title)
            .bind(&request.description)
            .bind(visible)
            .bind(sort_order)
            .execute(p)
            .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO permission_groups (name, title, description, visible, sort_order)
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(&request.name)
            .bind(&request.title)
            .bind(&request.description)
            .bind(visible)
            .bind(sort_order)
            .execute(p)
            .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO permission_groups (name, title, description, visible, sort_order)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .bind(&request.name)
            .bind(&request.title)
            .bind(&request.description)
            .bind(visible)
            .bind(sort_order)
            .execute(p)
            .await?;
        }
    }

    Ok(PermissionGroup {
        name: request.name,
        title: request.title,
        description: request.description,
        visible: request.visible.unwrap_or(true),
        sort_order,
    })
}

/// Delete a permission group
pub async fn delete_permission_group(
    pool: &DatabasePool,
    name: &str,
) -> Result<(), PermissionDatabaseError> {
    match pool {
        DatabasePool::MySql(p) => {
            let result = sqlx::query("DELETE FROM permission_groups WHERE name = ?")
                .bind(name)
                .execute(p)
                .await?;
            
            if result.rows_affected() == 0 {
                return Err(PermissionDatabaseError::PermissionGroupNotFound(name.to_string()));
            }
        }
        DatabasePool::Postgres(p) => {
            let result = sqlx::query("DELETE FROM permission_groups WHERE name = $1")
                .bind(name)
                .execute(p)
                .await?;
            
            if result.rows_affected() == 0 {
                return Err(PermissionDatabaseError::PermissionGroupNotFound(name.to_string()));
            }
        }
        DatabasePool::Sqlite(p) => {
            let result = sqlx::query("DELETE FROM permission_groups WHERE name = ?1")
                .bind(name)
                .execute(p)
                .await?;
            
            if result.rows_affected() == 0 {
                return Err(PermissionDatabaseError::PermissionGroupNotFound(name.to_string()));
            }
        }
    }

    Ok(())
}

/// List all permission groups
pub async fn list_permission_groups(
    pool: &DatabasePool,
) -> Result<Vec<PermissionGroup>, PermissionDatabaseError> {
    let mut groups = Vec::new();

    match pool {
        DatabasePool::MySql(p) => {
            let rows = sqlx::query("SELECT name, title, description, visible, sort_order FROM permission_groups ORDER BY sort_order, name")
                .fetch_all(p)
                .await?;
            
            for row in rows {
                let visible_int: i32 = row.get("visible");
                groups.push(PermissionGroup {
                    name: row.get("name"),
                    title: row.get("title"),
                    description: row.get("description"),
                    visible: visible_int != 0,
                    sort_order: row.get("sort_order"),
                });
            }
        }
        DatabasePool::Postgres(p) => {
            let rows = sqlx::query("SELECT name, title, description, visible, sort_order FROM permission_groups ORDER BY sort_order, name")
                .fetch_all(p)
                .await?;
            
            for row in rows {
                let visible_int: i32 = row.get("visible");
                groups.push(PermissionGroup {
                    name: row.get("name"),
                    title: row.get("title"),
                    description: row.get("description"),
                    visible: visible_int != 0,
                    sort_order: row.get("sort_order"),
                });
            }
        }
        DatabasePool::Sqlite(p) => {
            let rows = sqlx::query("SELECT name, title, description, visible, sort_order FROM permission_groups ORDER BY sort_order, name")
                .fetch_all(p)
                .await?;
            
            for row in rows {
                let visible_int: i32 = row.get("visible");
                groups.push(PermissionGroup {
                    name: row.get("name"),
                    title: row.get("title"),
                    description: row.get("description"),
                    visible: visible_int != 0,
                    sort_order: row.get("sort_order"),
                });
            }
        }
    }

    Ok(groups)
}

/// Create a new permission
pub async fn create_permission(
    pool: &DatabasePool,
    request: CreatePermissionRequest,
) -> Result<Permission, PermissionDatabaseError> {
    // Verify permission group exists
    let group_exists = match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM permission_groups WHERE name = ?")
                .bind(&request.permission_group_name)
                .fetch_one(p)
                .await?;
            let count: i64 = row.get("count");
            count > 0
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM permission_groups WHERE name = $1")
                .bind(&request.permission_group_name)
                .fetch_one(p)
                .await?;
            let count: i64 = row.get("count");
            count > 0
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM permission_groups WHERE name = ?1")
                .bind(&request.permission_group_name)
                .fetch_one(p)
                .await?;
            let count: i64 = row.get("count");
            count > 0
        }
    };

    if !group_exists {
        return Err(PermissionDatabaseError::PermissionGroupNotFound(
            request.permission_group_name,
        ));
    }

    let visible = request.visible.unwrap_or(true) as i32;
    let sort_order = request.sort_order.unwrap_or(0);

    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query(
                "INSERT INTO permissions (permission_group_name, name, title, description, visible, sort_order)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(&request.permission_group_name)
            .bind(&request.name)
            .bind(&request.title)
            .bind(&request.description)
            .bind(visible)
            .bind(sort_order)
            .execute(p)
            .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO permissions (permission_group_name, name, title, description, visible, sort_order)
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(&request.permission_group_name)
            .bind(&request.name)
            .bind(&request.title)
            .bind(&request.description)
            .bind(visible)
            .bind(sort_order)
            .execute(p)
            .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO permissions (permission_group_name, name, title, description, visible, sort_order)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .bind(&request.permission_group_name)
            .bind(&request.name)
            .bind(&request.title)
            .bind(&request.description)
            .bind(visible)
            .bind(sort_order)
            .execute(p)
            .await?;
        }
    }

    Ok(Permission {
        permission_group_name: request.permission_group_name,
        name: request.name,
        title: request.title,
        description: request.description,
        visible: request.visible.unwrap_or(true),
        sort_order,
    })
}

/// Delete a permission
pub async fn delete_permission(
    pool: &DatabasePool,
    name: &str,
) -> Result<(), PermissionDatabaseError> {
    match pool {
        DatabasePool::MySql(p) => {
            let result = sqlx::query("DELETE FROM permissions WHERE name = ?")
                .bind(name)
                .execute(p)
                .await?;
            
            if result.rows_affected() == 0 {
                return Err(PermissionDatabaseError::PermissionNotFound(name.to_string()));
            }
        }
        DatabasePool::Postgres(p) => {
            let result = sqlx::query("DELETE FROM permissions WHERE name = $1")
                .bind(name)
                .execute(p)
                .await?;
            
            if result.rows_affected() == 0 {
                return Err(PermissionDatabaseError::PermissionNotFound(name.to_string()));
            }
        }
        DatabasePool::Sqlite(p) => {
            let result = sqlx::query("DELETE FROM permissions WHERE name = ?1")
                .bind(name)
                .execute(p)
                .await?;
            
            if result.rows_affected() == 0 {
                return Err(PermissionDatabaseError::PermissionNotFound(name.to_string()));
            }
        }
    }

    Ok(())
}

/// List all permissions
pub async fn list_permissions(
    pool: &DatabasePool,
) -> Result<Vec<Permission>, PermissionDatabaseError> {
    let mut permissions = Vec::new();

    match pool {
        DatabasePool::MySql(p) => {
            let rows = sqlx::query("SELECT permission_group_name, name, title, description, visible, sort_order FROM permissions ORDER BY permission_group_name, sort_order, name")
                .fetch_all(p)
                .await?;
            
            for row in rows {
                let visible_int: i32 = row.get("visible");
                permissions.push(Permission {
                    permission_group_name: row.get("permission_group_name"),
                    name: row.get("name"),
                    title: row.get("title"),
                    description: row.get("description"),
                    visible: visible_int != 0,
                    sort_order: row.get("sort_order"),
                });
            }
        }
        DatabasePool::Postgres(p) => {
            let rows = sqlx::query("SELECT permission_group_name, name, title, description, visible, sort_order FROM permissions ORDER BY permission_group_name, sort_order, name")
                .fetch_all(p)
                .await?;
            
            for row in rows {
                let visible_int: i32 = row.get("visible");
                permissions.push(Permission {
                    permission_group_name: row.get("permission_group_name"),
                    name: row.get("name"),
                    title: row.get("title"),
                    description: row.get("description"),
                    visible: visible_int != 0,
                    sort_order: row.get("sort_order"),
                });
            }
        }
        DatabasePool::Sqlite(p) => {
            let rows = sqlx::query("SELECT permission_group_name, name, title, description, visible, sort_order FROM permissions ORDER BY permission_group_name, sort_order, name")
                .fetch_all(p)
                .await?;
            
            for row in rows {
                let visible_int: i32 = row.get("visible");
                permissions.push(Permission {
                    permission_group_name: row.get("permission_group_name"),
                    name: row.get("name"),
                    title: row.get("title"),
                    description: row.get("description"),
                    visible: visible_int != 0,
                    sort_order: row.get("sort_order"),
                });
            }
        }
    }

    Ok(permissions)
}

/// List all permissions for a user in a specific organization
pub async fn list_user_permissions(
    pool: &DatabasePool,
    user_id: &str,
    organization_uuid: &str,
) -> Result<Vec<UserPermission>, PermissionDatabaseError> {
    let mut user_permissions = Vec::new();

    match pool {
        DatabasePool::MySql(p) => {
            let rows = sqlx::query(
                "SELECT user_id, organization_uuid, permission_name, 
                        DATE_FORMAT(created_at, '%Y-%m-%d %H:%i:%s') as created_at
                 FROM user_permissions
                 WHERE user_id = ? AND organization_uuid = ?
                 ORDER BY created_at DESC",
            )
            .bind(user_id)
            .bind(organization_uuid)
            .fetch_all(p)
            .await?;
            
            for row in rows {
                user_permissions.push(UserPermission {
                    user_id: row.get("user_id"),
                    organization_uuid: row.get("organization_uuid"),
                    permission_name: row.get("permission_name"),
                    created_at: row.get("created_at"),
                });
            }
        }
        DatabasePool::Postgres(p) => {
            let rows = sqlx::query(
                "SELECT user_id, organization_uuid, permission_name, 
                        TO_CHAR(created_at, 'YYYY-MM-DD HH24:MI:SS') as created_at
                 FROM user_permissions
                 WHERE user_id = $1 AND organization_uuid = $2
                 ORDER BY created_at DESC",
            )
            .bind(user_id)
            .bind(organization_uuid)
            .fetch_all(p)
            .await?;
            
            for row in rows {
                user_permissions.push(UserPermission {
                    user_id: row.get("user_id"),
                    organization_uuid: row.get("organization_uuid"),
                    permission_name: row.get("permission_name"),
                    created_at: row.get("created_at"),
                });
            }
        }
        DatabasePool::Sqlite(p) => {
            let rows = sqlx::query(
                "SELECT user_id, organization_uuid, permission_name, 
                        strftime('%Y-%m-%d %H:%M:%S', created_at) as created_at
                 FROM user_permissions
                 WHERE user_id = ?1 AND organization_uuid = ?2
                 ORDER BY created_at DESC",
            )
            .bind(user_id)
            .bind(organization_uuid)
            .fetch_all(p)
            .await?;
            
            for row in rows {
                user_permissions.push(UserPermission {
                    user_id: row.get("user_id"),
                    organization_uuid: row.get("organization_uuid"),
                    permission_name: row.get("permission_name"),
                    created_at: row.get("created_at"),
                });
            }
        }
    }

    Ok(user_permissions)
}

/// Add a permission to a user for a specific organization
pub async fn add_user_permission(
    pool: &DatabasePool,
    user_id: &str,
    organization_uuid: &str,
    permission_name: &str,
) -> Result<(), PermissionDatabaseError> {
    // Verify permission exists
    let permission_exists = match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM permissions WHERE name = ?")
                .bind(permission_name)
                .fetch_one(p)
                .await?;
            let count: i64 = row.get("count");
            count > 0
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM permissions WHERE name = $1")
                .bind(permission_name)
                .fetch_one(p)
                .await?;
            let count: i64 = row.get("count");
            count > 0
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM permissions WHERE name = ?1")
                .bind(permission_name)
                .fetch_one(p)
                .await?;
            let count: i64 = row.get("count");
            count > 0
        }
    };

    if !permission_exists {
        return Err(PermissionDatabaseError::PermissionNotFound(permission_name.to_string()));
    }

    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query(
                "INSERT INTO user_permissions (user_id, organization_uuid, permission_name)
                 VALUES (?, ?, ?)
                 ON DUPLICATE KEY UPDATE permission_name = permission_name",
            )
            .bind(user_id)
            .bind(organization_uuid)
            .bind(permission_name)
            .execute(p)
            .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO user_permissions (user_id, organization_uuid, permission_name)
                 VALUES ($1, $2, $3)
                 ON CONFLICT (user_id, organization_uuid, permission_name) DO NOTHING",
            )
            .bind(user_id)
            .bind(organization_uuid)
            .bind(permission_name)
            .execute(p)
            .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query(
                "INSERT OR IGNORE INTO user_permissions (user_id, organization_uuid, permission_name)
                 VALUES (?1, ?2, ?3)",
            )
            .bind(user_id)
            .bind(organization_uuid)
            .bind(permission_name)
            .execute(p)
            .await?;
        }
    }

    Ok(())
}

/// Delete a specific permission from a user for a specific organization
pub async fn delete_user_permission(
    pool: &DatabasePool,
    user_id: &str,
    organization_uuid: &str,
    permission_name: &str,
) -> Result<(), PermissionDatabaseError> {
    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query(
                "DELETE FROM user_permissions
                 WHERE user_id = ? AND organization_uuid = ? AND permission_name = ?",
            )
            .bind(user_id)
            .bind(organization_uuid)
            .bind(permission_name)
            .execute(p)
            .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query(
                "DELETE FROM user_permissions
                 WHERE user_id = $1 AND organization_uuid = $2 AND permission_name = $3",
            )
            .bind(user_id)
            .bind(organization_uuid)
            .bind(permission_name)
            .execute(p)
            .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query(
                "DELETE FROM user_permissions
                 WHERE user_id = ?1 AND organization_uuid = ?2 AND permission_name = ?3",
            )
            .bind(user_id)
            .bind(organization_uuid)
            .bind(permission_name)
            .execute(p)
            .await?;
        }
    }

    Ok(())
}

/// Delete all permissions for a user in a specific organization
pub async fn delete_all_user_permissions(
    pool: &DatabasePool,
    user_id: &str,
    organization_uuid: &str,
) -> Result<(), PermissionDatabaseError> {
    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query(
                "DELETE FROM user_permissions
                 WHERE user_id = ? AND organization_uuid = ?",
            )
            .bind(user_id)
            .bind(organization_uuid)
            .execute(p)
            .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query(
                "DELETE FROM user_permissions
                 WHERE user_id = $1 AND organization_uuid = $2",
            )
            .bind(user_id)
            .bind(organization_uuid)
            .execute(p)
            .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query(
                "DELETE FROM user_permissions
                 WHERE user_id = ?1 AND organization_uuid = ?2",
            )
            .bind(user_id)
            .bind(organization_uuid)
            .execute(p)
            .await?;
        }
    }

    Ok(())
}

