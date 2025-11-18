//! Database operations for credentials management

use crate::credentials::credentials::CredentialsManager;
use crate::credentials::error::CredentialsError;
use crate::database::DatabasePool;
use crate::user::database::{user_belongs_to_organization, user_has_permission};
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::Row;
use uuid::Uuid;

/// Credential record (without encrypted data, for listing)
#[derive(Debug, Clone)]
pub struct CredentialMetadata {
    pub uuid: String,
    pub organization_uuid: String,
    pub name: String,
    pub credential_type: String,
    pub creator_user_uuid: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Credential with decrypted data
#[derive(Debug, Clone)]
pub struct Credential {
    pub uuid: String,
    pub organization_uuid: String,
    pub name: String,
    pub credential_type: String,
    pub data: Value, // Decrypted JSON data
    pub creator_user_uuid: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// List all credentials for an organization (without encrypted data)
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `organization_uuid` - UUID of the organization
/// * `user_uuid` - UUID of the user requesting the list
///
/// # Returns
/// Vector of credential metadata (without encrypted data)
///
/// # Errors
/// Returns `CredentialsError` if:
/// - User does not belong to organization
/// - User does not have permission to list credentials
/// - Database operation fails
pub async fn list_credentials(
    pool: &DatabasePool,
    organization_uuid: &str,
    user_uuid: &str,
) -> Result<Vec<CredentialMetadata>, CredentialsError> {
    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(pool, user_uuid, organization_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            CredentialsError::Database(e.into())
        })?;

    if !belongs {
        return Err(CredentialsError::UserNotInOrganization);
    }

    // Check permission (using a generic permission for now)
    // TODO: Define specific permission like "can_view_credentials"
    let has_permission = user_has_permission(
        pool,
        user_uuid,
        organization_uuid,
        "can_view_credentials",
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error checking permission: {}", e);
        CredentialsError::Database(e.into())
    })?;

    if !has_permission {
        return Err(CredentialsError::PermissionDenied);
    }

    // Query credentials
    match pool {
        DatabasePool::MySql(p) => {
            let rows = sqlx::query(
                "SELECT uuid, organization_uuid, name, type, creator_user_uuid, created_at, updated_at
                 FROM credentials
                 WHERE organization_uuid = ?
                 ORDER BY created_at DESC",
            )
            .bind(organization_uuid)
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .map(|row| CredentialMetadata {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    name: row.get("name"),
                    credential_type: row.get("type"),
                    creator_user_uuid: row.get("creator_user_uuid"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    updated_at: row.try_get("updated_at").ok().flatten(),
                })
                .collect())
        }
        DatabasePool::Postgres(p) => {
            let rows = sqlx::query(
                "SELECT uuid, organization_uuid, name, type, creator_user_uuid, created_at, updated_at
                 FROM credentials
                 WHERE organization_uuid = $1
                 ORDER BY created_at DESC",
            )
            .bind(organization_uuid)
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .map(|row| CredentialMetadata {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    name: row.get("name"),
                    credential_type: row.get("type"),
                    creator_user_uuid: row.get("creator_user_uuid"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    updated_at: row.try_get("updated_at").ok().flatten(),
                })
                .collect())
        }
        DatabasePool::Sqlite(p) => {
            let rows = sqlx::query(
                "SELECT uuid, organization_uuid, name, type, creator_user_uuid, created_at, updated_at
                 FROM credentials
                 WHERE organization_uuid = ?1
                 ORDER BY created_at DESC",
            )
            .bind(organization_uuid)
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .map(|row| CredentialMetadata {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    name: row.get("name"),
                    credential_type: row.get("type"),
                    creator_user_uuid: row.get("creator_user_uuid"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    updated_at: row.try_get("updated_at").ok().flatten(),
                })
                .collect())
        }
    }
}

/// Create a new credential
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `manager` - Credentials manager for encryption
/// * `organization_uuid` - UUID of the organization
/// * `user_uuid` - UUID of the user creating the credential
/// * `name` - Name of the credential
/// * `credential_type` - Type of credential (e.g., "chroma", "openai", "github")
/// * `data` - JSON data to encrypt and store
///
/// # Returns
/// UUID of the created credential
///
/// # Errors
/// Returns `CredentialsError` if:
/// - User does not belong to organization
/// - User does not have permission to create credentials
/// - Encryption fails
/// - Database operation fails
pub async fn create_credential(
    pool: &DatabasePool,
    manager: &CredentialsManager,
    organization_uuid: &str,
    user_uuid: &str,
    name: &str,
    credential_type: &str,
    data: &Value,
) -> Result<String, CredentialsError> {
    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(pool, user_uuid, organization_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            CredentialsError::Database(e.into())
        })?;

    if !belongs {
        return Err(CredentialsError::UserNotInOrganization);
    }

    // Check permission
    let has_permission = user_has_permission(
        pool,
        user_uuid,
        organization_uuid,
        "can_create_credentials",
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error checking permission: {}", e);
        CredentialsError::Database(e.into())
    })?;

    if !has_permission {
        return Err(CredentialsError::PermissionDenied);
    }

    // Generate UUID
    let credential_uuid = Uuid::new_v4().to_string();

    // Encrypt data
    let encrypted_data = manager.encrypt(data)?;

    // Insert into database
    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query(
                "INSERT INTO credentials (uuid, organization_uuid, name, type, encrypted_data, creator_user_uuid, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
            )
            .bind(&credential_uuid)
            .bind(organization_uuid)
            .bind(name)
            .bind(credential_type)
            .bind(&encrypted_data)
            .bind(user_uuid)
            .execute(p)
            .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO credentials (uuid, organization_uuid, name, type, encrypted_data, creator_user_uuid, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP)",
            )
            .bind(&credential_uuid)
            .bind(organization_uuid)
            .bind(name)
            .bind(credential_type)
            .bind(&encrypted_data)
            .bind(user_uuid)
            .execute(p)
            .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO credentials (uuid, organization_uuid, name, type, encrypted_data, creator_user_uuid, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)",
            )
            .bind(&credential_uuid)
            .bind(organization_uuid)
            .bind(name)
            .bind(credential_type)
            .bind(&encrypted_data)
            .bind(user_uuid)
            .execute(p)
            .await?;
        }
    }

    Ok(credential_uuid)
}

/// Get a credential by UUID (with decrypted data)
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `manager` - Credentials manager for decryption
/// * `credential_uuid` - UUID of the credential
/// * `organization_uuid` - UUID of the organization (for access control)
/// * `user_uuid` - UUID of the user requesting the credential
///
/// # Returns
/// Credential with decrypted data
///
/// # Errors
/// Returns `CredentialsError` if:
/// - Credential not found
/// - User does not belong to organization
/// - User does not have permission
/// - Decryption fails
/// - Database operation fails
pub async fn get_credential(
    pool: &DatabasePool,
    manager: &CredentialsManager,
    credential_uuid: &str,
    organization_uuid: &str,
    user_uuid: &str,
) -> Result<Credential, CredentialsError> {
    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(pool, user_uuid, organization_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            CredentialsError::Database(e.into())
        })?;

    if !belongs {
        return Err(CredentialsError::UserNotInOrganization);
    }

    // Check permission
    let has_permission = user_has_permission(
        pool,
        user_uuid,
        organization_uuid,
        "can_view_credentials",
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error checking permission: {}", e);
        CredentialsError::Database(e.into())
    })?;

    if !has_permission {
        return Err(CredentialsError::PermissionDenied);
    }

    // Query credential
    let (uuid, org_uuid, name, credential_type, encrypted_data, creator_uuid, created_at, updated_at) = match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query(
                "SELECT uuid, organization_uuid, name, type, encrypted_data, creator_user_uuid, created_at, updated_at
                 FROM credentials
                 WHERE uuid = ? AND organization_uuid = ?",
            )
            .bind(credential_uuid)
            .bind(organization_uuid)
            .fetch_optional(p)
            .await?;

            let row = row.ok_or_else(|| CredentialsError::CredentialNotFound(credential_uuid.to_string()))?;

            (
                row.get("uuid"),
                row.get("organization_uuid"),
                row.get("name"),
                row.get("type"),
                row.get::<Vec<u8>, _>("encrypted_data"),
                row.get("creator_user_uuid"),
                row.get::<DateTime<Utc>, _>("created_at"),
                row.try_get("updated_at").ok().flatten(),
            )
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT uuid, organization_uuid, name, type, encrypted_data, creator_user_uuid, created_at, updated_at
                 FROM credentials
                 WHERE uuid = $1 AND organization_uuid = $2",
            )
            .bind(credential_uuid)
            .bind(organization_uuid)
            .fetch_optional(p)
            .await?;

            let row = row.ok_or_else(|| CredentialsError::CredentialNotFound(credential_uuid.to_string()))?;

            (
                row.get("uuid"),
                row.get("organization_uuid"),
                row.get("name"),
                row.get("type"),
                row.get::<Vec<u8>, _>("encrypted_data"),
                row.get("creator_user_uuid"),
                row.get::<DateTime<Utc>, _>("created_at"),
                row.try_get("updated_at").ok().flatten(),
            )
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT uuid, organization_uuid, name, type, encrypted_data, creator_user_uuid, created_at, updated_at
                 FROM credentials
                 WHERE uuid = ?1 AND organization_uuid = ?2",
            )
            .bind(credential_uuid)
            .bind(organization_uuid)
            .fetch_optional(p)
            .await?;

            let row = row.ok_or_else(|| CredentialsError::CredentialNotFound(credential_uuid.to_string()))?;

            (
                row.get("uuid"),
                row.get("organization_uuid"),
                row.get("name"),
                row.get("type"),
                row.get::<Vec<u8>, _>("encrypted_data"),
                row.get("creator_user_uuid"),
                row.get::<DateTime<Utc>, _>("created_at"),
                row.try_get("updated_at").ok().flatten(),
            )
        }
    };

    // Decrypt data
    let data = manager.decrypt(&encrypted_data)?;

    Ok(Credential {
        uuid,
        organization_uuid: org_uuid,
        name,
        credential_type,
        data,
        creator_user_uuid: creator_uuid,
        created_at,
        updated_at,
    })
}

/// Get multiple credentials by UUIDs (with decrypted data)
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `manager` - Credentials manager for decryption
/// * `credential_uuids` - Vector of credential UUIDs
/// * `organization_uuid` - UUID of the organization (for access control)
/// * `user_uuid` - UUID of the user requesting the credentials
///
/// # Returns
/// Vector of credentials with decrypted data
///
/// # Errors
/// Returns `CredentialsError` if:
/// - User does not belong to organization
/// - User does not have permission
/// - Decryption fails
/// - Database operation fails
///
/// # Note
/// This function is used by workflows to retrieve credentials.
/// It does not require individual credential permissions, but requires
/// organization membership and general credential viewing permission.
pub async fn get_credentials(
    pool: &DatabasePool,
    manager: &CredentialsManager,
    credential_uuids: &[String],
    organization_uuid: &str,
    user_uuid: &str,
) -> Result<Vec<Credential>, CredentialsError> {
    if credential_uuids.is_empty() {
        return Ok(Vec::new());
    }

    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(pool, user_uuid, organization_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            CredentialsError::Database(e.into())
        })?;

    if !belongs {
        return Err(CredentialsError::UserNotInOrganization);
    }

    // Check permission
    let has_permission = user_has_permission(
        pool,
        user_uuid,
        organization_uuid,
        "can_view_credentials",
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error checking permission: {}", e);
        CredentialsError::Database(e.into())
    })?;

    if !has_permission {
        return Err(CredentialsError::PermissionDenied);
    }

    // Query credentials
    match pool {
        DatabasePool::MySql(p) => {
            // Build query with IN clause
            let placeholders = credential_uuids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let query = format!(
                "SELECT uuid, organization_uuid, name, type, encrypted_data, creator_user_uuid, created_at, updated_at
                 FROM credentials
                 WHERE uuid IN ({}) AND organization_uuid = ?",
                placeholders
            );

            let mut query_builder = sqlx::query(&query);
            for uuid in credential_uuids {
                query_builder = query_builder.bind(uuid);
            }
            query_builder = query_builder.bind(organization_uuid);

            let rows = query_builder.fetch_all(p).await?;

            let mut credentials = Vec::new();
            for row in rows {
                let encrypted_data: Vec<u8> = row.get("encrypted_data");
                let data = manager.decrypt(&encrypted_data)?;

                credentials.push(Credential {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    name: row.get("name"),
                    credential_type: row.get("type"),
                    data,
                    creator_user_uuid: row.get("creator_user_uuid"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    updated_at: row.try_get("updated_at").ok().flatten(),
                });
            }

            Ok(credentials)
        }
        DatabasePool::Postgres(p) => {
            // PostgreSQL uses $1, $2, etc.
            let placeholders: Vec<String> = (1..=credential_uuids.len())
                .map(|i| format!("${}", i))
                .collect();
            let query = format!(
                "SELECT uuid, organization_uuid, name, type, encrypted_data, creator_user_uuid, created_at, updated_at
                 FROM credentials
                 WHERE uuid IN ({}) AND organization_uuid = ${}",
                placeholders.join(","),
                credential_uuids.len() + 1
            );

            let mut query_builder = sqlx::query(&query);
            for uuid in credential_uuids {
                query_builder = query_builder.bind(uuid);
            }
            query_builder = query_builder.bind(organization_uuid);

            let rows = query_builder.fetch_all(p).await?;

            let mut credentials = Vec::new();
            for row in rows {
                let encrypted_data: Vec<u8> = row.get("encrypted_data");
                let data = manager.decrypt(&encrypted_data)?;

                credentials.push(Credential {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    name: row.get("name"),
                    credential_type: row.get("type"),
                    data,
                    creator_user_uuid: row.get("creator_user_uuid"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    updated_at: row.try_get("updated_at").ok().flatten(),
                });
            }

            Ok(credentials)
        }
        DatabasePool::Sqlite(p) => {
            // SQLite uses ?1, ?2, etc.
            let placeholders: Vec<String> = (1..=credential_uuids.len())
                .map(|i| format!("?{}", i))
                .collect();
            let query = format!(
                "SELECT uuid, organization_uuid, name, type, encrypted_data, creator_user_uuid, created_at, updated_at
                 FROM credentials
                 WHERE uuid IN ({}) AND organization_uuid = ?{}",
                placeholders.join(","),
                credential_uuids.len() + 1
            );

            let mut query_builder = sqlx::query(&query);
            for uuid in credential_uuids {
                query_builder = query_builder.bind(uuid);
            }
            query_builder = query_builder.bind(organization_uuid);

            let rows = query_builder.fetch_all(p).await?;

            let mut credentials = Vec::new();
            for row in rows {
                let encrypted_data: Vec<u8> = row.get("encrypted_data");
                let data = manager.decrypt(&encrypted_data)?;

                credentials.push(Credential {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    name: row.get("name"),
                    credential_type: row.get("type"),
                    data,
                    creator_user_uuid: row.get("creator_user_uuid"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    updated_at: row.try_get("updated_at").ok().flatten(),
                });
            }

            Ok(credentials)
        }
    }
}

/// Update a credential (overwrites encrypted data)
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `manager` - Credentials manager for encryption
/// * `credential_uuid` - UUID of the credential to update
/// * `organization_uuid` - UUID of the organization
/// * `user_uuid` - UUID of the user updating the credential
/// * `name` - New name (optional)
/// * `data` - New data to encrypt and store
///
/// # Returns
/// Success
///
/// # Errors
/// Returns `CredentialsError` if:
/// - Credential not found
/// - User does not belong to organization
/// - User does not have permission
/// - Encryption fails
/// - Database operation fails
pub async fn update_credential(
    pool: &DatabasePool,
    manager: &CredentialsManager,
    credential_uuid: &str,
    organization_uuid: &str,
    user_uuid: &str,
    name: Option<&str>,
    data: &Value,
) -> Result<(), CredentialsError> {
    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(pool, user_uuid, organization_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            CredentialsError::Database(e.into())
        })?;

    if !belongs {
        return Err(CredentialsError::UserNotInOrganization);
    }

    // Check permission
    let has_permission = user_has_permission(
        pool,
        user_uuid,
        organization_uuid,
        "can_edit_credentials",
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error checking permission: {}", e);
        CredentialsError::Database(e.into())
    })?;

    if !has_permission {
        return Err(CredentialsError::PermissionDenied);
    }

    // Encrypt new data
    let encrypted_data = manager.encrypt(data)?;

    // Update in database
    match pool {
        DatabasePool::MySql(p) => {
            if let Some(new_name) = name {
                sqlx::query(
                    "UPDATE credentials
                     SET name = ?, encrypted_data = ?, updated_at = CURRENT_TIMESTAMP
                     WHERE uuid = ? AND organization_uuid = ?",
                )
                .bind(new_name)
                .bind(&encrypted_data)
                .bind(credential_uuid)
                .bind(organization_uuid)
                .execute(p)
                .await?;
            } else {
                sqlx::query(
                    "UPDATE credentials
                     SET encrypted_data = ?, updated_at = CURRENT_TIMESTAMP
                     WHERE uuid = ? AND organization_uuid = ?",
                )
                .bind(&encrypted_data)
                .bind(credential_uuid)
                .bind(organization_uuid)
                .execute(p)
                .await?;
            }
        }
        DatabasePool::Postgres(p) => {
            if let Some(new_name) = name {
                sqlx::query(
                    "UPDATE credentials
                     SET name = $1, encrypted_data = $2, updated_at = CURRENT_TIMESTAMP
                     WHERE uuid = $3 AND organization_uuid = $4",
                )
                .bind(new_name)
                .bind(&encrypted_data)
                .bind(credential_uuid)
                .bind(organization_uuid)
                .execute(p)
                .await?;
            } else {
                sqlx::query(
                    "UPDATE credentials
                     SET encrypted_data = $1, updated_at = CURRENT_TIMESTAMP
                     WHERE uuid = $2 AND organization_uuid = $3",
                )
                .bind(&encrypted_data)
                .bind(credential_uuid)
                .bind(organization_uuid)
                .execute(p)
                .await?;
            }
        }
        DatabasePool::Sqlite(p) => {
            if let Some(new_name) = name {
                sqlx::query(
                    "UPDATE credentials
                     SET name = ?1, encrypted_data = ?2, updated_at = CURRENT_TIMESTAMP
                     WHERE uuid = ?3 AND organization_uuid = ?4",
                )
                .bind(new_name)
                .bind(&encrypted_data)
                .bind(credential_uuid)
                .bind(organization_uuid)
                .execute(p)
                .await?;
            } else {
                sqlx::query(
                    "UPDATE credentials
                     SET encrypted_data = ?1, updated_at = CURRENT_TIMESTAMP
                     WHERE uuid = ?2 AND organization_uuid = ?3",
                )
                .bind(&encrypted_data)
                .bind(credential_uuid)
                .bind(organization_uuid)
                .execute(p)
                .await?;
            }
        }
    }

    Ok(())
}

/// Delete a credential
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `credential_uuid` - UUID of the credential to delete
/// * `organization_uuid` - UUID of the organization
/// * `user_uuid` - UUID of the user deleting the credential
///
/// # Returns
/// Success
///
/// # Errors
/// Returns `CredentialsError` if:
/// - Credential not found
/// - User does not belong to organization
/// - User does not have permission
/// - Database operation fails
pub async fn delete_credential(
    pool: &DatabasePool,
    credential_uuid: &str,
    organization_uuid: &str,
    user_uuid: &str,
) -> Result<(), CredentialsError> {
    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(pool, user_uuid, organization_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            CredentialsError::Database(e.into())
        })?;

    if !belongs {
        return Err(CredentialsError::UserNotInOrganization);
    }

    // Check permission
    let has_permission = user_has_permission(
        pool,
        user_uuid,
        organization_uuid,
        "can_delete_credentials",
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error checking permission: {}", e);
        CredentialsError::Database(e.into())
    })?;

    if !has_permission {
        return Err(CredentialsError::PermissionDenied);
    }

    // Delete from database
    match pool {
        DatabasePool::MySql(p) => {
            let result = sqlx::query(
                "DELETE FROM credentials
                 WHERE uuid = ? AND organization_uuid = ?",
            )
            .bind(credential_uuid)
            .bind(organization_uuid)
            .execute(p)
            .await?;

            if result.rows_affected() == 0 {
                return Err(CredentialsError::CredentialNotFound(credential_uuid.to_string()));
            }
        }
        DatabasePool::Postgres(p) => {
            let result = sqlx::query(
                "DELETE FROM credentials
                 WHERE uuid = $1 AND organization_uuid = $2",
            )
            .bind(credential_uuid)
            .bind(organization_uuid)
            .execute(p)
            .await?;

            if result.rows_affected() == 0 {
                return Err(CredentialsError::CredentialNotFound(credential_uuid.to_string()));
            }
        }
        DatabasePool::Sqlite(p) => {
            let result = sqlx::query(
                "DELETE FROM credentials
                 WHERE uuid = ?1 AND organization_uuid = ?2",
            )
            .bind(credential_uuid)
            .bind(organization_uuid)
            .execute(p)
            .await?;

            if result.rows_affected() == 0 {
                return Err(CredentialsError::CredentialNotFound(credential_uuid.to_string()));
            }
        }
    }

    Ok(())
}

