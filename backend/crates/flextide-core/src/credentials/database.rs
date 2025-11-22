//! Database operations for credentials management

use crate::credentials::credentials::CredentialsManager;
use crate::credentials::error::CredentialsError;
use crate::database::DatabasePool;
use crate::user::{user_belongs_to_organization, user_has_permission};
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

    // Check permission
    let has_permission = user_has_permission(
        pool,
        user_uuid,
        organization_uuid,
        "can_see_all_credentials",
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
                "SELECT uuid, organization_uuid, name, credential_type, creator_user_uuid, created_at, updated_at
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
                    credential_type: row.get("credential_type"),
                    creator_user_uuid: row.get("creator_user_uuid"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    updated_at: row.try_get("updated_at").ok().flatten(),
                })
                .collect())
        }
        DatabasePool::Postgres(p) => {
            let rows = sqlx::query(
                "SELECT uuid, organization_uuid, name, credential_type, creator_user_uuid, created_at, updated_at
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
                    credential_type: row.get("credential_type"),
                    creator_user_uuid: row.get("creator_user_uuid"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    updated_at: row.try_get("updated_at").ok().flatten(),
                })
                .collect())
        }
        DatabasePool::Sqlite(p) => {
            let rows = sqlx::query(
                "SELECT uuid, organization_uuid, name, credential_type, creator_user_uuid, created_at, updated_at
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
                    credential_type: row.get("credential_type"),
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
                "INSERT INTO credentials (uuid, organization_uuid, name, credential_type, encrypted_data, creator_user_uuid, created_at)
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
                "INSERT INTO credentials (uuid, organization_uuid, name, credential_type, encrypted_data, creator_user_uuid, created_at)
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
                "INSERT INTO credentials (uuid, organization_uuid, name, credential_type, encrypted_data, creator_user_uuid, created_at)
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
        "can_see_all_credentials",
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
                "SELECT uuid, organization_uuid, name, credential_type, encrypted_data, creator_user_uuid, created_at, updated_at
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
                row.get("credential_type"),
                row.get::<Vec<u8>, _>("encrypted_data"),
                row.get("creator_user_uuid"),
                row.get::<DateTime<Utc>, _>("created_at"),
                row.try_get("updated_at").ok().flatten(),
            )
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT uuid, organization_uuid, name, credential_type, encrypted_data, creator_user_uuid, created_at, updated_at
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
                row.get("credential_type"),
                row.get::<Vec<u8>, _>("encrypted_data"),
                row.get("creator_user_uuid"),
                row.get::<DateTime<Utc>, _>("created_at"),
                row.try_get("updated_at").ok().flatten(),
            )
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT uuid, organization_uuid, name, credential_type, encrypted_data, creator_user_uuid, created_at, updated_at
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
                row.get("credential_type"),
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
        "can_see_all_credentials",
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
                "SELECT uuid, organization_uuid, name, credential_type, encrypted_data, creator_user_uuid, created_at, updated_at
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
                    credential_type: row.get("credential_type"),
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
                "SELECT uuid, organization_uuid, name, credential_type, encrypted_data, creator_user_uuid, created_at, updated_at
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
                    credential_type: row.get("credential_type"),
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
                "SELECT uuid, organization_uuid, name, credential_type, encrypted_data, creator_user_uuid, created_at, updated_at
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
                    credential_type: row.get("credential_type"),
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

/// Get all credentials of a specific type for an organization (with decrypted data)
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `manager` - Credentials manager for decryption
/// * `organization_uuid` - UUID of the organization
/// * `credential_type` - Type of credential to filter by (e.g., "openai_credential", "github_credential")
///
/// # Returns
/// Vector of credentials with decrypted data, filtered by credential type
///
/// # Errors
/// Returns `CredentialsError` if:
/// - Decryption fails
/// - Database operation fails
///
/// # Note
/// This function is useful for workflows and worker nodes that need all credentials of a specific type.
/// It does not perform permission checks, as it may be called from worker nodes without user context.
pub async fn get_credentials_by_type(
    pool: &DatabasePool,
    manager: &CredentialsManager,
    organization_uuid: &str,
    credential_type: &str,
) -> Result<Vec<Credential>, CredentialsError> {

    // Query credentials by type
    match pool {
        DatabasePool::MySql(p) => {
            let rows = sqlx::query(
                "SELECT uuid, organization_uuid, name, credential_type, encrypted_data, creator_user_uuid, created_at, updated_at
                 FROM credentials
                 WHERE organization_uuid = ? AND credential_type = ?
                 ORDER BY created_at DESC",
            )
            .bind(organization_uuid)
            .bind(credential_type)
            .fetch_all(p)
            .await?;

            let mut credentials = Vec::new();
            for row in rows {
                let encrypted_data: Vec<u8> = row.get("encrypted_data");
                let data = manager.decrypt(&encrypted_data)?;

                credentials.push(Credential {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    name: row.get("name"),
                    credential_type: row.get("credential_type"),
                    data,
                    creator_user_uuid: row.get("creator_user_uuid"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    updated_at: row.try_get("updated_at").ok().flatten(),
                });
            }

            Ok(credentials)
        }
        DatabasePool::Postgres(p) => {
            let rows = sqlx::query(
                "SELECT uuid, organization_uuid, name, credential_type, encrypted_data, creator_user_uuid, created_at, updated_at
                 FROM credentials
                 WHERE organization_uuid = $1 AND credential_type = $2
                 ORDER BY created_at DESC",
            )
            .bind(organization_uuid)
            .bind(credential_type)
            .fetch_all(p)
            .await?;

            let mut credentials = Vec::new();
            for row in rows {
                let encrypted_data: Vec<u8> = row.get("encrypted_data");
                let data = manager.decrypt(&encrypted_data)?;

                credentials.push(Credential {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    name: row.get("name"),
                    credential_type: row.get("credential_type"),
                    data,
                    creator_user_uuid: row.get("creator_user_uuid"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    updated_at: row.try_get("updated_at").ok().flatten(),
                });
            }

            Ok(credentials)
        }
        DatabasePool::Sqlite(p) => {
            let rows = sqlx::query(
                "SELECT uuid, organization_uuid, name, credential_type, encrypted_data, creator_user_uuid, created_at, updated_at
                 FROM credentials
                 WHERE organization_uuid = ?1 AND credential_type = ?2
                 ORDER BY created_at DESC",
            )
            .bind(organization_uuid)
            .bind(credential_type)
            .fetch_all(p)
            .await?;

            let mut credentials = Vec::new();
            for row in rows {
                let encrypted_data: Vec<u8> = row.get("encrypted_data");
                let data = manager.decrypt(&encrypted_data)?;

                credentials.push(Credential {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    name: row.get("name"),
                    credential_type: row.get("credential_type"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::create_test_pool;
    use hex;
    use serde_json::json;
    use uuid::Uuid;

    /// Set up test database with required tables
    async fn setup_test_db() -> DatabasePool {
        let pool = create_test_pool().await.expect("Failed to create test pool");

        // Create users table
        match &pool {
            DatabasePool::Sqlite(p) => {
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS users (
                        uuid CHAR(36) NOT NULL PRIMARY KEY,
                        email VARCHAR(255) NOT NULL UNIQUE,
                        password_hash TEXT NOT NULL,
                        salt VARCHAR(255),
                        prename VARCHAR(255) NOT NULL,
                        lastname VARCHAR(255),
                        created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        mail_verified INTEGER NOT NULL DEFAULT 0,
                        activated INTEGER NOT NULL DEFAULT 1
                    )",
                )
                .execute(p)
                .await
                .expect("Failed to create users table");

                // Create organizations table
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS organizations (
                        uuid CHAR(36) NOT NULL PRIMARY KEY,
                        name VARCHAR(255) NOT NULL,
                        owner_user_id CHAR(36) NOT NULL,
                        created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
                    )",
                )
                .execute(p)
                .await
                .expect("Failed to create organizations table");

                // Create organization_members table
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS organization_members (
                        org_id CHAR(36) NOT NULL,
                        user_id CHAR(36) NOT NULL,
                        role VARCHAR(20) NOT NULL DEFAULT 'member',
                        joined_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        PRIMARY KEY (org_id, user_id)
                    )",
                )
                .execute(p)
                .await
                .expect("Failed to create organization_members table");

                // Create permission_groups table
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS permission_groups (
                        id CHAR(36) NOT NULL PRIMARY KEY,
                        name VARCHAR(255) NOT NULL UNIQUE,
                        title VARCHAR(255) NOT NULL,
                        description TEXT,
                        visible INTEGER NOT NULL DEFAULT 1,
                        sort_order INTEGER NOT NULL DEFAULT 0
                    )",
                )
                .execute(p)
                .await
                .expect("Failed to create permission_groups table");

                // Create permissions table
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS permissions (
                        id CHAR(36) NOT NULL PRIMARY KEY,
                        permission_group_name VARCHAR(255) NOT NULL,
                        name VARCHAR(255) NOT NULL UNIQUE,
                        title VARCHAR(255) NOT NULL,
                        description TEXT,
                        visible INTEGER NOT NULL DEFAULT 1,
                        sort_order INTEGER NOT NULL DEFAULT 0,
                        FOREIGN KEY (permission_group_name) REFERENCES permission_groups(name) ON DELETE RESTRICT
                    )",
                )
                .execute(p)
                .await
                .expect("Failed to create permissions table");

                // Create user_permissions table
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS user_permissions (
                        user_id CHAR(36) NOT NULL,
                        organization_uuid CHAR(36) NOT NULL,
                        permission_name VARCHAR(255) NOT NULL,
                        created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        PRIMARY KEY (user_id, organization_uuid, permission_name),
                        FOREIGN KEY (user_id) REFERENCES users(uuid) ON DELETE CASCADE,
                        FOREIGN KEY (organization_uuid) REFERENCES organizations(uuid) ON DELETE CASCADE,
                        FOREIGN KEY (permission_name) REFERENCES permissions(name) ON DELETE CASCADE
                    )",
                )
                .execute(p)
                .await
                .expect("Failed to create user_permissions table");

                // Create credentials table
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS credentials (
                        uuid CHAR(36) NOT NULL PRIMARY KEY,
                        organization_uuid CHAR(36) NOT NULL,
                        name VARCHAR(255) NOT NULL,
                        type VARCHAR(255) NOT NULL,
                        encrypted_data BLOB NOT NULL,
                        creator_user_uuid CHAR(36) NOT NULL,
                        created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        updated_at TIMESTAMP NULL,
                        FOREIGN KEY (organization_uuid) REFERENCES organizations(uuid) ON DELETE CASCADE,
                        FOREIGN KEY (creator_user_uuid) REFERENCES users(uuid) ON DELETE RESTRICT
                    )",
                )
                .execute(p)
                .await
                .expect("Failed to create credentials table");

                // Insert super_admin permission group and permission
                sqlx::query(
                    "INSERT OR IGNORE INTO permission_groups (id, name, title, description, visible, sort_order)
                     VALUES ('00000000-0000-0000-0000-000000000005', 'super_admin', 'Super Admin', 'Super administrator permissions', 1, 0)",
                )
                .execute(p)
                .await
                .expect("Failed to insert permission group");

                sqlx::query(
                    "INSERT OR IGNORE INTO permissions (id, permission_group_name, name, title, description, visible, sort_order)
                     VALUES ('00000000-0000-0000-0000-000000000001', 'super_admin', 'super_admin', 'Super Admin', 'Super administrator permission', 1, 0)",
                )
                .execute(p)
                .await
                .expect("Failed to insert permission");

                // Insert credential permissions
                for (perm_id, perm_name, perm_title) in [
                    ("00000000-0000-0000-0000-000000000010", "can_see_all_credentials", "View Credentials"),
                    ("00000000-0000-0000-0000-000000000011", "can_create_credentials", "Create Credentials"),
                    ("00000000-0000-0000-0000-000000000012", "can_edit_credentials", "Edit Credentials"),
                    ("00000000-0000-0000-0000-000000000013", "can_delete_credentials", "Delete Credentials"),
                ] {
                    sqlx::query(
                        "INSERT OR IGNORE INTO permissions (id, permission_group_name, name, title, description, visible, sort_order)
                         VALUES (?1, 'super_admin', ?2, ?3, 'Credential permission', 1, 0)",
                    )
                    .bind(perm_id)
                    .bind(perm_name)
                    .bind(perm_title)
                    .execute(p)
                    .await
                    .expect("Failed to insert credential permission");
                }
            }
            _ => unreachable!("Test pool should be SQLite"),
        }

        pool
    }

    /// Create a test user
    async fn create_test_user(pool: &DatabasePool, email: &str) -> String {
        let user_uuid = Uuid::new_v4().to_string();
        match pool {
            DatabasePool::Sqlite(p) => {
                sqlx::query(
                    "INSERT INTO users (uuid, email, password_hash, prename, mail_verified, activated)
                     VALUES (?1, ?2, ?3, ?4, 1, 1)",
                )
                .bind(&user_uuid)
                .bind(email)
                .bind("hashed_password")
                .bind("Test")
                .execute(p)
                .await
                .expect("Failed to create test user");
            }
            _ => unreachable!("Test pool should be SQLite"),
        }
        user_uuid
    }

    /// Create a test organization
    async fn create_test_organization(pool: &DatabasePool, owner_user_uuid: &str) -> String {
        let org_uuid = Uuid::new_v4().to_string();
        match pool {
            DatabasePool::Sqlite(p) => {
                sqlx::query(
                    "INSERT INTO organizations (uuid, name, owner_user_id)
                     VALUES (?1, ?2, ?3)",
                )
                .bind(&org_uuid)
                .bind("Test Organization")
                .bind(owner_user_uuid)
                .execute(p)
                .await
                .expect("Failed to create test organization");
            }
            _ => unreachable!("Test pool should be SQLite"),
        }
        org_uuid
    }

    /// Add user to organization
    async fn add_user_to_organization(pool: &DatabasePool, user_uuid: &str, org_uuid: &str) {
        match pool {
            DatabasePool::Sqlite(p) => {
                sqlx::query(
                    "INSERT INTO organization_members (org_id, user_id, role)
                     VALUES (?1, ?2, 'member')",
                )
                .bind(org_uuid)
                .bind(user_uuid)
                .execute(p)
                .await
                .expect("Failed to add user to organization");
            }
            _ => unreachable!("Test pool should be SQLite"),
        }
    }

    /// Grant permission to user
    async fn grant_permission(pool: &DatabasePool, user_uuid: &str, org_uuid: &str, permission: &str) {
        match pool {
            DatabasePool::Sqlite(p) => {
                sqlx::query(
                    "INSERT OR IGNORE INTO user_permissions (user_id, organization_uuid, permission_name)
                     VALUES (?1, ?2, ?3)",
                )
                .bind(user_uuid)
                .bind(org_uuid)
                .bind(permission)
                .execute(p)
                .await
                .expect("Failed to grant permission");
            }
            _ => unreachable!("Test pool should be SQLite"),
        }
    }

    fn create_test_manager() -> CredentialsManager {
        let test_key = hex::encode([0u8; 32]);
        std::env::set_var("CREDENTIALS_MASTER_KEY", test_key);
        CredentialsManager::new().unwrap()
    }

    #[tokio::test]
    async fn test_create_credential() {
        let pool = setup_test_db().await;
        let manager = create_test_manager();
        let user_uuid = create_test_user(&pool, "test@example.com").await;
        let org_uuid = create_test_organization(&pool, &user_uuid).await;
        add_user_to_organization(&pool, &user_uuid, &org_uuid).await;
        grant_permission(&pool, &user_uuid, &org_uuid, "can_create_credentials").await;

        let data = json!({"api_key": "test-key-123"});
        let credential_uuid = create_credential(
            &pool,
            &manager,
            &org_uuid,
            &user_uuid,
            "Test Credential",
            "openai",
            &data,
        )
        .await
        .unwrap();

        assert!(!credential_uuid.is_empty());
    }

    #[tokio::test]
    async fn test_create_credential_user_not_in_org() {
        let pool = setup_test_db().await;
        let manager = create_test_manager();
        let user_uuid = create_test_user(&pool, "test@example.com").await;
        let org_uuid = create_test_organization(&pool, &user_uuid).await;
        // Don't add user to organization

        let data = json!({"api_key": "test-key-123"});
        let result = create_credential(
            &pool,
            &manager,
            &org_uuid,
            &user_uuid,
            "Test Credential",
            "openai",
            &data,
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            CredentialsError::UserNotInOrganization => {}
            _ => panic!("Expected UserNotInOrganization error"),
        }
    }

    #[tokio::test]
    async fn test_create_credential_no_permission() {
        let pool = setup_test_db().await;
        let manager = create_test_manager();
        let user_uuid = create_test_user(&pool, "test@example.com").await;
        let org_uuid = create_test_organization(&pool, &user_uuid).await;
        add_user_to_organization(&pool, &user_uuid, &org_uuid).await;
        // Don't grant permission

        let data = json!({"api_key": "test-key-123"});
        let result = create_credential(
            &pool,
            &manager,
            &org_uuid,
            &user_uuid,
            "Test Credential",
            "openai",
            &data,
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            CredentialsError::PermissionDenied => {}
            _ => panic!("Expected PermissionDenied error"),
        }
    }

    #[tokio::test]
    async fn test_list_credentials() {
        let pool = setup_test_db().await;
        let manager = create_test_manager();
        let user_uuid = create_test_user(&pool, "test@example.com").await;
        let org_uuid = create_test_organization(&pool, &user_uuid).await;
        add_user_to_organization(&pool, &user_uuid, &org_uuid).await;
        grant_permission(&pool, &user_uuid, &org_uuid, "can_see_all_credentials").await;
        grant_permission(&pool, &user_uuid, &org_uuid, "can_create_credentials").await;

        // Create some credentials
        let data1 = json!({"api_key": "key1"});
        let data2 = json!({"api_key": "key2"});
        create_credential(
            &pool,
            &manager,
            &org_uuid,
            &user_uuid,
            "Credential 1",
            "openai",
            &data1,
        )
        .await
        .unwrap();
        create_credential(
            &pool,
            &manager,
            &org_uuid,
            &user_uuid,
            "Credential 2",
            "github",
            &data2,
        )
        .await
        .unwrap();

        let credentials = list_credentials(&pool, &org_uuid, &user_uuid)
            .await
            .unwrap();

        assert_eq!(credentials.len(), 2);
        assert!(credentials.iter().any(|c| c.name == "Credential 1"));
        assert!(credentials.iter().any(|c| c.name == "Credential 2"));
    }

    #[tokio::test]
    async fn test_list_credentials_empty() {
        let pool = setup_test_db().await;
        let user_uuid = create_test_user(&pool, "test@example.com").await;
        let org_uuid = create_test_organization(&pool, &user_uuid).await;
        add_user_to_organization(&pool, &user_uuid, &org_uuid).await;
        grant_permission(&pool, &user_uuid, &org_uuid, "can_see_all_credentials").await;

        let credentials = list_credentials(&pool, &org_uuid, &user_uuid)
            .await
            .unwrap();

        assert_eq!(credentials.len(), 0);
    }

    #[tokio::test]
    async fn test_get_credential() {
        let pool = setup_test_db().await;
        let manager = create_test_manager();
        let user_uuid = create_test_user(&pool, "test@example.com").await;
        let org_uuid = create_test_organization(&pool, &user_uuid).await;
        add_user_to_organization(&pool, &user_uuid, &org_uuid).await;
        grant_permission(&pool, &user_uuid, &org_uuid, "can_see_all_credentials").await;
        grant_permission(&pool, &user_uuid, &org_uuid, "can_create_credentials").await;

        let original_data = json!({"api_key": "test-key-123", "base_url": "https://api.example.com"});
        let credential_uuid = create_credential(
            &pool,
            &manager,
            &org_uuid,
            &user_uuid,
            "Test Credential",
            "openai",
            &original_data,
        )
        .await
        .unwrap();

        let credential = get_credential(&pool, &manager, &credential_uuid, &org_uuid, &user_uuid)
            .await
            .unwrap();

        assert_eq!(credential.uuid, credential_uuid);
        assert_eq!(credential.name, "Test Credential");
        assert_eq!(credential.credential_type, "openai");
        assert_eq!(credential.data, original_data);
    }

    #[tokio::test]
    async fn test_get_credential_not_found() {
        let pool = setup_test_db().await;
        let manager = create_test_manager();
        let user_uuid = create_test_user(&pool, "test@example.com").await;
        let org_uuid = create_test_organization(&pool, &user_uuid).await;
        add_user_to_organization(&pool, &user_uuid, &org_uuid).await;
        grant_permission(&pool, &user_uuid, &org_uuid, "can_see_all_credentials").await;

        let fake_uuid = Uuid::new_v4().to_string();
        let result = get_credential(&pool, &manager, &fake_uuid, &org_uuid, &user_uuid).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            CredentialsError::CredentialNotFound(_) => {}
            _ => panic!("Expected CredentialNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_get_credentials_multiple() {
        let pool = setup_test_db().await;
        let manager = create_test_manager();
        let user_uuid = create_test_user(&pool, "test@example.com").await;
        let org_uuid = create_test_organization(&pool, &user_uuid).await;
        add_user_to_organization(&pool, &user_uuid, &org_uuid).await;
        grant_permission(&pool, &user_uuid, &org_uuid, "can_see_all_credentials").await;
        grant_permission(&pool, &user_uuid, &org_uuid, "can_create_credentials").await;

        let data1 = json!({"api_key": "key1"});
        let data2 = json!({"api_key": "key2"});
        let data3 = json!({"api_key": "key3"});

        let uuid1 = create_credential(
            &pool,
            &manager,
            &org_uuid,
            &user_uuid,
            "Credential 1",
            "openai",
            &data1,
        )
        .await
        .unwrap();
        let uuid2 = create_credential(
            &pool,
            &manager,
            &org_uuid,
            &user_uuid,
            "Credential 2",
            "github",
            &data2,
        )
        .await
        .unwrap();
        let uuid3 = create_credential(
            &pool,
            &manager,
            &org_uuid,
            &user_uuid,
            "Credential 3",
            "openai",
            &data3,
        )
        .await
        .unwrap();

        let credentials = get_credentials(
            &pool,
            &manager,
            &[uuid1.clone(), uuid2.clone(), uuid3.clone()],
            &org_uuid,
            &user_uuid,
        )
        .await
        .unwrap();

        assert_eq!(credentials.len(), 3);
        assert!(credentials.iter().any(|c| c.uuid == uuid1 && c.data == data1));
        assert!(credentials.iter().any(|c| c.uuid == uuid2 && c.data == data2));
        assert!(credentials.iter().any(|c| c.uuid == uuid3 && c.data == data3));
    }

    #[tokio::test]
    async fn test_get_credentials_empty_list() {
        let pool = setup_test_db().await;
        let manager = create_test_manager();
        let user_uuid = create_test_user(&pool, "test@example.com").await;
        let org_uuid = create_test_organization(&pool, &user_uuid).await;
        add_user_to_organization(&pool, &user_uuid, &org_uuid).await;
        grant_permission(&pool, &user_uuid, &org_uuid, "can_see_all_credentials").await;

        let credentials = get_credentials(&pool, &manager, &[], &org_uuid, &user_uuid)
            .await
            .unwrap();

        assert_eq!(credentials.len(), 0);
    }

    #[tokio::test]
    async fn test_update_credential() {
        let pool = setup_test_db().await;
        let manager = create_test_manager();
        let user_uuid = create_test_user(&pool, "test@example.com").await;
        let org_uuid = create_test_organization(&pool, &user_uuid).await;
        add_user_to_organization(&pool, &user_uuid, &org_uuid).await;
        grant_permission(&pool, &user_uuid, &org_uuid, "can_create_credentials").await;
        grant_permission(&pool, &user_uuid, &org_uuid, "can_edit_credentials").await;

        let original_data = json!({"api_key": "old-key"});
        let credential_uuid = create_credential(
            &pool,
            &manager,
            &org_uuid,
            &user_uuid,
            "Test Credential",
            "openai",
            &original_data,
        )
        .await
        .unwrap();

        let updated_data = json!({"api_key": "new-key", "base_url": "https://api.example.com"});
        update_credential(
            &pool,
            &manager,
            &credential_uuid,
            &org_uuid,
            &user_uuid,
            Some("Updated Name"),
            &updated_data,
        )
        .await
        .unwrap();

        let credential = get_credential(&pool, &manager, &credential_uuid, &org_uuid, &user_uuid)
            .await
            .unwrap();

        assert_eq!(credential.name, "Updated Name");
        assert_eq!(credential.data, updated_data);
    }

    #[tokio::test]
    async fn test_update_credential_no_name_change() {
        let pool = setup_test_db().await;
        let manager = create_test_manager();
        let user_uuid = create_test_user(&pool, "test@example.com").await;
        let org_uuid = create_test_organization(&pool, &user_uuid).await;
        add_user_to_organization(&pool, &user_uuid, &org_uuid).await;
        grant_permission(&pool, &user_uuid, &org_uuid, "can_create_credentials").await;
        grant_permission(&pool, &user_uuid, &org_uuid, "can_edit_credentials").await;

        let original_data = json!({"api_key": "old-key"});
        let credential_uuid = create_credential(
            &pool,
            &manager,
            &org_uuid,
            &user_uuid,
            "Test Credential",
            "openai",
            &original_data,
        )
        .await
        .unwrap();

        let updated_data = json!({"api_key": "new-key"});
        update_credential(
            &pool,
            &manager,
            &credential_uuid,
            &org_uuid,
            &user_uuid,
            None, // Don't change name
            &updated_data,
        )
        .await
        .unwrap();

        let credential = get_credential(&pool, &manager, &credential_uuid, &org_uuid, &user_uuid)
            .await
            .unwrap();

        assert_eq!(credential.name, "Test Credential"); // Name unchanged
        assert_eq!(credential.data, updated_data);
    }

    #[tokio::test]
    async fn test_delete_credential() {
        let pool = setup_test_db().await;
        let manager = create_test_manager();
        let user_uuid = create_test_user(&pool, "test@example.com").await;
        let org_uuid = create_test_organization(&pool, &user_uuid).await;
        add_user_to_organization(&pool, &user_uuid, &org_uuid).await;
        grant_permission(&pool, &user_uuid, &org_uuid, "can_create_credentials").await;
        grant_permission(&pool, &user_uuid, &org_uuid, "can_delete_credentials").await;

        let data = json!({"api_key": "test-key"});
        let credential_uuid = create_credential(
            &pool,
            &manager,
            &org_uuid,
            &user_uuid,
            "Test Credential",
            "openai",
            &data,
        )
        .await
        .unwrap();

        delete_credential(&pool, &credential_uuid, &org_uuid, &user_uuid)
            .await
            .unwrap();

        // Verify it's deleted
        let result = get_credential(&pool, &manager, &credential_uuid, &org_uuid, &user_uuid).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CredentialsError::CredentialNotFound(_) => {}
            _ => panic!("Expected CredentialNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_delete_credential_not_found() {
        let pool = setup_test_db().await;
        let user_uuid = create_test_user(&pool, "test@example.com").await;
        let org_uuid = create_test_organization(&pool, &user_uuid).await;
        add_user_to_organization(&pool, &user_uuid, &org_uuid).await;
        grant_permission(&pool, &user_uuid, &org_uuid, "can_delete_credentials").await;

        let fake_uuid = Uuid::new_v4().to_string();
        let result = delete_credential(&pool, &fake_uuid, &org_uuid, &user_uuid).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            CredentialsError::CredentialNotFound(_) => {}
            _ => panic!("Expected CredentialNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_credential_isolation_between_organizations() {
        let pool = setup_test_db().await;
        let manager = create_test_manager();
        
        // Create two organizations
        let user1_uuid = create_test_user(&pool, "user1@example.com").await;
        let user2_uuid = create_test_user(&pool, "user2@example.com").await;
        let org1_uuid = create_test_organization(&pool, &user1_uuid).await;
        let org2_uuid = create_test_organization(&pool, &user2_uuid).await;
        
        add_user_to_organization(&pool, &user1_uuid, &org1_uuid).await;
        add_user_to_organization(&pool, &user2_uuid, &org2_uuid).await;
        grant_permission(&pool, &user1_uuid, &org1_uuid, "can_create_credentials").await;
        grant_permission(&pool, &user1_uuid, &org1_uuid, "can_see_all_credentials").await;
        grant_permission(&pool, &user2_uuid, &org2_uuid, "can_create_credentials").await;
        grant_permission(&pool, &user2_uuid, &org2_uuid, "can_see_all_credentials").await;

        // Create credential in org1
        let data1 = json!({"api_key": "org1-key"});
        let credential1_uuid = create_credential(
            &pool,
            &manager,
            &org1_uuid,
            &user1_uuid,
            "Org1 Credential",
            "openai",
            &data1,
        )
        .await
        .unwrap();

        // Create credential in org2
        let data2 = json!({"api_key": "org2-key"});
        let credential2_uuid = create_credential(
            &pool,
            &manager,
            &org2_uuid,
            &user2_uuid,
            "Org2 Credential",
            "openai",
            &data2,
        )
        .await
        .unwrap();

        // User1 should only see org1 credentials
        let org1_credentials = list_credentials(&pool, &org1_uuid, &user1_uuid).await.unwrap();
        assert_eq!(org1_credentials.len(), 1);
        assert_eq!(org1_credentials[0].uuid, credential1_uuid);

        // User2 should only see org2 credentials
        let org2_credentials = list_credentials(&pool, &org2_uuid, &user2_uuid).await.unwrap();
        assert_eq!(org2_credentials.len(), 1);
        assert_eq!(org2_credentials[0].uuid, credential2_uuid);

        // User1 cannot access org2 credential
        let result = get_credential(&pool, &manager, &credential2_uuid, &org1_uuid, &user1_uuid).await;
        assert!(result.is_err());
    }
}

