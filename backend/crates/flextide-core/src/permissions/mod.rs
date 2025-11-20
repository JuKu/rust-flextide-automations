//! Permission management module
//! 
//! Provides functionality for managing permissions, permission groups, and user permissions.

mod database;

pub use database::{
    create_permission_group, delete_permission_group, list_permission_groups,
    create_permission, delete_permission, list_permissions,
    list_user_permissions, add_user_permission, delete_user_permission, delete_all_user_permissions,
    PermissionDatabaseError,
};

use serde::{Deserialize, Serialize};

/// Permission group data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGroup {
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub visible: bool,
    pub sort_order: i32,
}

/// Permission data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub permission_group_name: String,
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub visible: bool,
    pub sort_order: i32,
}

/// User permission data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermission {
    pub user_id: String,
    pub organization_uuid: String,
    pub permission_name: String,
    pub created_at: String,
}

/// Create permission group request
#[derive(Debug, Deserialize)]
pub struct CreatePermissionGroupRequest {
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub visible: Option<bool>,
    pub sort_order: Option<i32>,
}

/// Create permission request
#[derive(Debug, Deserialize)]
pub struct CreatePermissionRequest {
    pub permission_group_name: String,
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub visible: Option<bool>,
    pub sort_order: Option<i32>,
}

