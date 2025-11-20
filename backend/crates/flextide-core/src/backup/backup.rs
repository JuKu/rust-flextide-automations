//! Backup data structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Backup status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BackupStatus {
    Completed,
    Failed,
    InProgress,
    Cancelled,
}

impl From<&str> for BackupStatus {
    fn from(s: &str) -> Self {
        match s {
            "COMPLETED" => BackupStatus::Completed,
            "FAILED" => BackupStatus::Failed,
            "IN_PROGRESS" => BackupStatus::InProgress,
            "CANCELLED" => BackupStatus::Cancelled,
            _ => BackupStatus::Failed,
        }
    }
}

impl From<BackupStatus> for String {
    fn from(status: BackupStatus) -> Self {
        match status {
            BackupStatus::Completed => "COMPLETED".to_string(),
            BackupStatus::Failed => "FAILED".to_string(),
            BackupStatus::InProgress => "IN_PROGRESS".to_string(),
            BackupStatus::Cancelled => "CANCELLED".to_string(),
        }
    }
}

/// Backup record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backup {
    pub uuid: String,
    pub filename: String,
    pub full_path: String,
    pub creator_user_uuid: String,
    pub target_location: String,
    pub backup_status: BackupStatus,
    pub backup_hash_checksum: Option<String>,
    pub is_encrypted: bool,
    pub encryption_algorithm: Option<String>,
    pub encryption_master_key_name: Option<String>,
    pub start_timestamp: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    /// Whether the backup file still exists on the file system
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_exists: Option<bool>,
}

/// Backup job record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupJob {
    pub uuid: String,
    pub job_type: String,
    pub job_title: String,
    pub json_data: Option<Value>,
    pub last_execution_timestamp: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Paginated backup list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedBackups {
    pub backups: Vec<Backup>,
    pub total: u64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

/// Create backup request
#[derive(Debug, Deserialize)]
pub struct CreateBackupRequest {
    pub filename: String,
    pub target_location: Option<String>,
}

/// Create backup job request
#[derive(Debug, Deserialize)]
pub struct CreateBackupJobRequest {
    pub job_type: String,
    pub job_title: String,
    pub json_data: Option<Value>,
}

/// Update backup job request
#[derive(Debug, Deserialize)]
pub struct UpdateBackupJobRequest {
    pub job_title: Option<String>,
    pub json_data: Option<Value>,
}

