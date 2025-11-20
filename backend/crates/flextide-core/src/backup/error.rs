//! Error types for backup operations

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BackupError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("User does not belong to organization")]
    UserNotInOrganization,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Backup not found")]
    BackupNotFound,

    #[error("Backup job not found")]
    BackupJobNotFound,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid JSON data: {0}")]
    InvalidJson(#[from] serde_json::Error),

    #[error("Invalid backup status: {0}")]
    InvalidBackupStatus(String),

    #[error("Invalid job type: {0}")]
    InvalidJobType(String),
}

