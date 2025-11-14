//! Database operations for user management

use crate::database::{DatabaseError, DatabasePool};
use crate::user::{hash_password, User, UserCreationError};
use sqlx::Row;
use uuid::Uuid;

/// Error type for user database operations
#[derive(Debug, thiserror::Error)]
pub enum UserDatabaseError {
    #[error("User creation failed: {0}")]
    UserCreation(#[from] UserCreationError),

    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("SQL execution error: {0}")]
    Sql(#[from] sqlx::Error),
}

/// Check if there are any users in the database
///
/// # Errors
/// Returns `UserDatabaseError` if the database query fails
pub async fn has_any_users(pool: &DatabasePool) -> Result<bool, UserDatabaseError> {
    let count: i64 = match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM users")
                .fetch_one(p)
                .await?;
            row.get("count")
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM users")
                .fetch_one(p)
                .await?;
            row.get("count")
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM users")
                .fetch_one(p)
                .await?;
            row.get("count")
        }
    };

    Ok(count > 0)
}

/// Check if a user has a specific permission for an organization
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_uuid` - UUID of the user to check
/// * `organization_uuid` - UUID of the organization
/// * `permission` - Permission string to check (e.g., "module_crm_can_create_customers")
///
/// # Returns
/// Returns `true` if the user has the permission, `false` otherwise
///
/// # Errors
/// Returns `UserDatabaseError` if the database query fails
///
/// # Note
/// Users with the "super_admin" permission automatically have access to everything.
pub async fn user_has_permission(
    pool: &DatabasePool,
    user_uuid: &str,
    organization_uuid: &str,
    permission: &str,
) -> Result<bool, UserDatabaseError> {
    // First check if user has super_admin permission (grants access to everything)
    let has_super_admin = match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query(
                "SELECT COUNT(*) as count FROM user_permissions
                 WHERE user_id = ? AND organization_uuid = ? AND permission_name = 'super_admin'",
            )
            .bind(user_uuid)
            .bind(organization_uuid)
            .fetch_one(p)
            .await?;
            let count: i64 = row.get("count");
            count > 0
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT COUNT(*) as count FROM user_permissions
                 WHERE user_id = $1 AND organization_uuid = $2 AND permission_name = 'super_admin'",
            )
            .bind(user_uuid)
            .bind(organization_uuid)
            .fetch_one(p)
            .await?;
            let count: i64 = row.get("count");
            count > 0
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT COUNT(*) as count FROM user_permissions
                 WHERE user_id = ?1 AND organization_uuid = ?2 AND permission_name = 'super_admin'",
            )
            .bind(user_uuid)
            .bind(organization_uuid)
            .fetch_one(p)
            .await?;
            let count: i64 = row.get("count");
            count > 0
        }
    };

    if has_super_admin {
        return Ok(true);
    }

    // Check for the specific permission
    match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query(
                "SELECT COUNT(*) as count FROM user_permissions
                 WHERE user_id = ? AND organization_uuid = ? AND permission_name = ?",
            )
            .bind(user_uuid)
            .bind(organization_uuid)
            .bind(permission)
            .fetch_one(p)
            .await?;
            let count: i64 = row.get("count");
            Ok(count > 0)
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT COUNT(*) as count FROM user_permissions
                 WHERE user_id = $1 AND organization_uuid = $2 AND permission_name = $3",
            )
            .bind(user_uuid)
            .bind(organization_uuid)
            .bind(permission)
            .fetch_one(p)
            .await?;
            let count: i64 = row.get("count");
            Ok(count > 0)
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT COUNT(*) as count FROM user_permissions
                 WHERE user_id = ?1 AND organization_uuid = ?2 AND permission_name = ?3",
            )
            .bind(user_uuid)
            .bind(organization_uuid)
            .bind(permission)
            .fetch_one(p)
            .await?;
            let count: i64 = row.get("count");
            Ok(count > 0)
        }
    }
}

/// Get a user by email from the database
///
/// # Errors
/// Returns `UserDatabaseError` if the database query fails or user is not found
pub async fn get_user_by_email(pool: &DatabasePool, email: &str) -> Result<User, UserDatabaseError> {
    match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query(
                "SELECT uuid, email, password_hash, salt, prename, lastname, mail_verified, activated 
                 FROM users WHERE email = ?"
            )
            .bind(email)
            .fetch_optional(p)
            .await?;
            
            match row {
                Some(row) => {
                    let mail_verified_int: i32 = row.get("mail_verified");
                    let activated_int: i32 = row.get("activated");
                    
                    Ok(User {
                        uuid: row.get("uuid"),
                        email: row.get("email"),
                        password_hash: row.get("password_hash"),
                        salt: row.get::<Option<String>, _>("salt"),
                        prename: row.get("prename"),
                        lastname: row.get::<Option<String>, _>("lastname"),
                        mail_verified: mail_verified_int != 0,
                        activated: activated_int != 0,
                    })
                }
                None => Err(UserDatabaseError::Sql(sqlx::Error::RowNotFound)),
            }
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT uuid, email, password_hash, salt, prename, lastname, mail_verified, activated 
                 FROM users WHERE email = $1"
            )
            .bind(email)
            .fetch_optional(p)
            .await?;
            
            match row {
                Some(row) => {
                    let mail_verified_int: i32 = row.get("mail_verified");
                    let activated_int: i32 = row.get("activated");
                    
                    Ok(User {
                        uuid: row.get("uuid"),
                        email: row.get("email"),
                        password_hash: row.get("password_hash"),
                        salt: row.get::<Option<String>, _>("salt"),
                        prename: row.get("prename"),
                        lastname: row.get::<Option<String>, _>("lastname"),
                        mail_verified: mail_verified_int != 0,
                        activated: activated_int != 0,
                    })
                }
                None => Err(UserDatabaseError::Sql(sqlx::Error::RowNotFound)),
            }
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT uuid, email, password_hash, salt, prename, lastname, mail_verified, activated 
                 FROM users WHERE email = ?1"
            )
            .bind(email)
            .fetch_optional(p)
            .await?;
            
            match row {
                Some(row) => {
                    let mail_verified_int: i32 = row.get("mail_verified");
                    let activated_int: i32 = row.get("activated");
                    
                    Ok(User {
                        uuid: row.get("uuid"),
                        email: row.get("email"),
                        password_hash: row.get("password_hash"),
                        salt: row.get::<Option<String>, _>("salt"),
                        prename: row.get("prename"),
                        lastname: row.get::<Option<String>, _>("lastname"),
                        mail_verified: mail_verified_int != 0,
                        activated: activated_int != 0,
                    })
                }
                None => Err(UserDatabaseError::Sql(sqlx::Error::RowNotFound)),
            }
        }
    }
}

/// Create a default admin user if no users exist
///
/// Creates a user with:
/// - Email: admin@example.com
/// - Password: admin
/// - Prename: Admin
/// - Activated: true
/// - Mail verified: true (for admin user)
///
/// Also ensures the admin user has an organization:
/// - If the admin user doesn't belong to any organization, creates "My Organization"
/// - Adds the admin user as owner of the organization
///
/// # Errors
/// Returns `UserDatabaseError` if database operations fail
pub async fn ensure_default_admin_user(pool: &DatabasePool) -> Result<(), UserDatabaseError> {
    // Check if any users exist
    let admin_user_uuid = if has_any_users(pool).await? {
        // Users exist, check if admin user exists and get their UUID
        match get_user_by_email(pool, "admin@example.com").await {
            Ok(user) => user.uuid,
            Err(_) => {
                // Admin user doesn't exist, but other users do - don't create admin
                return Ok(());
            }
        }
    } else {
        // No users exist, create admin user
        let uuid = Uuid::new_v4().to_string();
        let email = "admin@example.com";
        let password = "admin";
        let password_hash = hash_password(password)
            .map_err(|e| UserDatabaseError::UserCreation(UserCreationError::PasswordHashing(e)))?;
        let prename = "Admin";
        let mail_verified = 1; // true for admin
        let activated = 1; // true

        match pool {
            DatabasePool::MySql(p) => {
                sqlx::query(
                    "INSERT INTO users (uuid, email, password_hash, salt, prename, lastname, mail_verified, activated) 
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(&uuid)
                .bind(email)
                .bind(&password_hash)
                .bind::<Option<String>>(None)
                .bind(prename)
                .bind::<Option<String>>(None)
                .bind(mail_verified)
                .bind(activated)
                .execute(p)
                .await?;
            }
            DatabasePool::Postgres(p) => {
                sqlx::query(
                    "INSERT INTO users (uuid, email, password_hash, salt, prename, lastname, mail_verified, activated) 
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                )
                .bind(&uuid)
                .bind(email)
                .bind(&password_hash)
                .bind::<Option<String>>(None)
                .bind(prename)
                .bind::<Option<String>>(None)
                .bind(mail_verified)
                .bind(activated)
                .execute(p)
                .await?;
            }
            DatabasePool::Sqlite(p) => {
                sqlx::query(
                    "INSERT INTO users (uuid, email, password_hash, salt, prename, lastname, mail_verified, activated) 
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                )
                .bind(&uuid)
                .bind(email)
                .bind(&password_hash)
                .bind::<Option<String>>(None)
                .bind(prename)
                .bind::<Option<String>>(None)
                .bind(mail_verified)
                .bind(activated)
                .execute(p)
                .await?;
            }
        }
        
        uuid
    };

    // Check if admin user belongs to any organization
    let has_organization = match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM organization_members WHERE user_id = ?")
                .bind(&admin_user_uuid)
                .fetch_one(p)
                .await?;
            let count: i64 = row.get("count");
            count > 0
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM organization_members WHERE user_id = $1")
                .bind(&admin_user_uuid)
                .fetch_one(p)
                .await?;
            let count: i64 = row.get("count");
            count > 0
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM organization_members WHERE user_id = ?1")
                .bind(&admin_user_uuid)
                .fetch_one(p)
                .await?;
            let count: i64 = row.get("count");
            count > 0
        }
    };

    // If admin user doesn't have an organization, create one
    if !has_organization {
        let org_uuid = Uuid::new_v4().to_string();
        let org_name = "My Organization";

        // Create organization
        match pool {
            DatabasePool::MySql(p) => {
                sqlx::query("INSERT INTO organizations (uuid, name, owner_user_id) VALUES (?, ?, ?)")
                    .bind(&org_uuid)
                    .bind(org_name)
                    .bind(&admin_user_uuid)
                    .execute(p)
                    .await?;
            }
            DatabasePool::Postgres(p) => {
                sqlx::query("INSERT INTO organizations (uuid, name, owner_user_id) VALUES ($1, $2, $3)")
                    .bind(&org_uuid)
                    .bind(org_name)
                    .bind(&admin_user_uuid)
                    .execute(p)
                    .await?;
            }
            DatabasePool::Sqlite(p) => {
                sqlx::query("INSERT INTO organizations (uuid, name, owner_user_id) VALUES (?1, ?2, ?3)")
                    .bind(&org_uuid)
                    .bind(org_name)
                    .bind(&admin_user_uuid)
                    .execute(p)
                    .await?;
            }
        }

        // Add admin user as owner of the organization
        match pool {
            DatabasePool::MySql(p) => {
                sqlx::query("INSERT INTO organization_members (org_id, user_id, role) VALUES (?, ?, ?)")
                    .bind(&org_uuid)
                    .bind(&admin_user_uuid)
                    .bind("owner")
                    .execute(p)
                    .await?;
            }
            DatabasePool::Postgres(p) => {
                sqlx::query("INSERT INTO organization_members (org_id, user_id, role) VALUES ($1, $2, $3)")
                    .bind(&org_uuid)
                    .bind(&admin_user_uuid)
                    .bind("owner")
                    .execute(p)
                    .await?;
            }
            DatabasePool::Sqlite(p) => {
                sqlx::query("INSERT INTO organization_members (org_id, user_id, role) VALUES (?1, ?2, ?3)")
                    .bind(&org_uuid)
                    .bind(&admin_user_uuid)
                    .bind("owner")
                    .execute(p)
                    .await?;
            }
        }

        // Grant super_admin permission to admin user for the organization
        match pool {
            DatabasePool::MySql(p) => {
                sqlx::query(
                    "INSERT INTO user_permissions (user_id, organization_uuid, permission_name)
                     VALUES (?, ?, 'super_admin')
                     ON DUPLICATE KEY UPDATE permission_name = permission_name",
                )
                .bind(&admin_user_uuid)
                .bind(&org_uuid)
                .execute(p)
                .await?;
            }
            DatabasePool::Postgres(p) => {
                sqlx::query(
                    "INSERT INTO user_permissions (user_id, organization_uuid, permission_name)
                     VALUES ($1, $2, 'super_admin')
                     ON CONFLICT (user_id, organization_uuid, permission_name) DO NOTHING",
                )
                .bind(&admin_user_uuid)
                .bind(&org_uuid)
                .execute(p)
                .await?;
            }
            DatabasePool::Sqlite(p) => {
                sqlx::query(
                    "INSERT OR IGNORE INTO user_permissions (user_id, organization_uuid, permission_name)
                     VALUES (?1, ?2, 'super_admin')",
                )
                .bind(&admin_user_uuid)
                .bind(&org_uuid)
                .execute(p)
                .await?;
            }
        }
    }

    Ok(())
}

/// Check if a user belongs to a specific organization
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_uuid` - UUID of the user to check
/// * `organization_uuid` - UUID of the organization to check
///
/// # Returns
/// Returns `true` if the user belongs to the organization, `false` otherwise
///
/// # Errors
/// Returns `UserDatabaseError` if the database query fails
pub async fn user_belongs_to_organization(
    pool: &DatabasePool,
    user_uuid: &str,
    organization_uuid: &str,
) -> Result<bool, UserDatabaseError> {
    match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query(
                "SELECT COUNT(*) as count FROM organization_members 
                 WHERE user_id = ? AND org_id = ?",
            )
            .bind(user_uuid)
            .bind(organization_uuid)
            .fetch_one(p)
            .await?;
            
            let count: i64 = row.get("count");
            Ok(count > 0)
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT COUNT(*) as count FROM organization_members 
                 WHERE user_id = $1 AND org_id = $2",
            )
            .bind(user_uuid)
            .bind(organization_uuid)
            .fetch_one(p)
            .await?;
            
            let count: i64 = row.get("count");
            Ok(count > 0)
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT COUNT(*) as count FROM organization_members 
                 WHERE user_id = ?1 AND org_id = ?2",
            )
            .bind(user_uuid)
            .bind(organization_uuid)
            .fetch_one(p)
            .await?;
            
            let count: i64 = row.get("count");
            Ok(count > 0)
        }
    }
}

