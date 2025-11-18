//! Error types for credentials management

use crate::database::DatabaseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CredentialsError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("SQL execution error: {0}")]
    Sql(#[from] sqlx::Error),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("Master key not found in environment variable CREDENTIALS_MASTER_KEY")]
    MasterKeyNotFound,

    #[error("Invalid master key format (must be 64 hex characters for 32-byte key)")]
    InvalidMasterKeyFormat,

    #[error("Credential not found: {0}")]
    CredentialNotFound(String),

    #[error("User does not belong to organization")]
    UserNotInOrganization,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid credential data format")]
    InvalidDataFormat,
}

