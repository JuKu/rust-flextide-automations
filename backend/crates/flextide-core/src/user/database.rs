//! Database operations for user management

use crate::database::{DatabaseError, DatabasePool};
use crate::user::{hash_password, User, UserCreationError};
use sqlx::Row;

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
/// # Errors
/// Returns `UserDatabaseError` if database operations fail
pub async fn ensure_default_admin_user(pool: &DatabasePool) -> Result<(), UserDatabaseError> {
    // Check if any users exist
    if has_any_users(pool).await? {
        return Ok(());
    }

    // Create admin user
    let uuid = uuid::Uuid::new_v4().to_string();
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

    Ok(())
}

