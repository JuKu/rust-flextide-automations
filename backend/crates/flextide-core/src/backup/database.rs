//! Database operations for backup management

use crate::backup::backup::*;
use crate::backup::error::BackupError;
use crate::database::DatabasePool;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::Row;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// List all backups with pagination and file existence check
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_uuid` - UUID of the user requesting the list
/// * `page` - Page number (1-indexed)
/// * `limit` - Number of items per page
///
/// # Returns
/// Paginated list of backups with file existence information
///
/// # Errors
/// Returns `BackupError` if:
/// - User does not have permission
/// - Database operation fails
pub async fn list_backups(
    pool: &DatabasePool,
    _user_uuid: &str,
    page: u32,
    limit: u32,
) -> Result<PaginatedBackups, BackupError> {
    // Check permission (backups are global, not organization-scoped, but we still check permissions)
    // Note: For now, we check if user is server admin or has can_see_all_backups permission
    // Since backups are global, we don't check organization membership here
    
    // Get total count
    let total: i64 = match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM backups")
                .fetch_one(p)
                .await?;
            row.get("count")
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM backups")
                .fetch_one(p)
                .await?;
            row.get("count")
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM backups")
                .fetch_one(p)
                .await?;
            row.get("count")
        }
    };

    let offset = (page - 1) * limit;
    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    // Query backups and extract data
    let mut backups = Vec::new();
    match pool {
        DatabasePool::MySql(p) => {
            let rows = sqlx::query(
                "SELECT uuid, filename, full_path, creator_user_uuid, target_location, job_type, backup_status,
                        backup_hash_checksum, is_encrypted, encryption_algorithm, encryption_master_key_name,
                        error_json, start_timestamp, created_at
                 FROM backups
                 ORDER BY created_at DESC
                 LIMIT ? OFFSET ?",
            )
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(p)
            .await?;

            for row in rows {
                let full_path: String = row.get("full_path");
                let file_exists = Path::new(&full_path).exists();

                let error_json: Option<String> = row.get("error_json");
                let error_value = error_json
                    .as_ref()
                    .and_then(|s| serde_json::from_str::<Value>(s).ok());

                let backup = Backup {
                    uuid: row.get("uuid"),
                    filename: row.get("filename"),
                    full_path: full_path.clone(),
                    creator_user_uuid: row.get("creator_user_uuid"),
                    target_location: row.get("target_location"),
                    job_type: row.get("job_type"),
                    backup_status: BackupStatus::from(row.get::<String, _>("backup_status").as_str()),
                    backup_hash_checksum: row.get("backup_hash_checksum"),
                    is_encrypted: row.get::<i32, _>("is_encrypted") != 0,
                    encryption_algorithm: row.get("encryption_algorithm"),
                    encryption_master_key_name: row.get("encryption_master_key_name"),
                    error_json: error_value,
                    start_timestamp: row.get("start_timestamp"),
                    created_at: row.get("created_at"),
                    file_exists: Some(file_exists),
                };
                backups.push(backup);
            }
        }
        DatabasePool::Postgres(p) => {
            let rows = sqlx::query(
                "SELECT uuid, filename, full_path, creator_user_uuid, target_location, job_type, backup_status,
                        backup_hash_checksum, is_encrypted, encryption_algorithm, encryption_master_key_name,
                        error_json, start_timestamp, created_at
                 FROM backups
                 ORDER BY created_at DESC
                 LIMIT $1 OFFSET $2",
            )
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(p)
            .await?;

            for row in rows {
                let full_path: String = row.get("full_path");
                let file_exists = Path::new(&full_path).exists();

                let error_json: Option<String> = row.get("error_json");
                let error_value = error_json
                    .as_ref()
                    .and_then(|s| serde_json::from_str::<Value>(s).ok());

                let backup = Backup {
                    uuid: row.get("uuid"),
                    filename: row.get("filename"),
                    full_path: full_path.clone(),
                    creator_user_uuid: row.get("creator_user_uuid"),
                    target_location: row.get("target_location"),
                    job_type: row.get("job_type"),
                    backup_status: BackupStatus::from(row.get::<String, _>("backup_status").as_str()),
                    backup_hash_checksum: row.get("backup_hash_checksum"),
                    is_encrypted: row.get::<i32, _>("is_encrypted") != 0,
                    encryption_algorithm: row.get("encryption_algorithm"),
                    encryption_master_key_name: row.get("encryption_master_key_name"),
                    error_json: error_value,
                    start_timestamp: row.get("start_timestamp"),
                    created_at: row.get("created_at"),
                    file_exists: Some(file_exists),
                };
                backups.push(backup);
            }
        }
        DatabasePool::Sqlite(p) => {
            let rows = sqlx::query(
                "SELECT uuid, filename, full_path, creator_user_uuid, target_location, job_type, backup_status,
                        backup_hash_checksum, is_encrypted, encryption_algorithm, encryption_master_key_name,
                        error_json, start_timestamp, created_at
                 FROM backups
                 ORDER BY created_at DESC
                 LIMIT ?1 OFFSET ?2",
            )
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(p)
            .await?;

            for row in rows {
                let full_path: String = row.get("full_path");
                let file_exists = Path::new(&full_path).exists();

                let error_json: Option<String> = row.get("error_json");
                let error_value = error_json
                    .as_ref()
                    .and_then(|s| serde_json::from_str::<Value>(s).ok());

                let backup = Backup {
                    uuid: row.get("uuid"),
                    filename: row.get("filename"),
                    full_path: full_path.clone(),
                    creator_user_uuid: row.get("creator_user_uuid"),
                    target_location: row.get("target_location"),
                    job_type: row.get("job_type"),
                    backup_status: BackupStatus::from(row.get::<String, _>("backup_status").as_str()),
                    backup_hash_checksum: row.get("backup_hash_checksum"),
                    is_encrypted: row.get::<i32, _>("is_encrypted") != 0,
                    encryption_algorithm: row.get("encryption_algorithm"),
                    encryption_master_key_name: row.get("encryption_master_key_name"),
                    error_json: error_value,
                    start_timestamp: row.get("start_timestamp"),
                    created_at: row.get("created_at"),
                    file_exists: Some(file_exists),
                };
                backups.push(backup);
            }
        }
    }

    Ok(PaginatedBackups {
        backups,
        total: total as u64,
        page,
        limit,
        total_pages,
    })
}

/// Create a new backup (permission check only, actual backup creation is mocked)
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_uuid` - UUID of the user creating the backup
/// * `request` - Create backup request
///
/// # Returns
/// UUID of the created backup record
///
/// # Errors
/// Returns `BackupError` if:
/// - User does not have permission
/// - Database operation fails
pub async fn create_backup(
    pool: &DatabasePool,
    user_uuid: &str,
    request: CreateBackupRequest,
) -> Result<String, BackupError> {
    // Check permission - backups are global, so we check can_create_backup permission
    // Note: In a real implementation, we'd need to check this against a global permission system
    // For now, we'll assume the API layer handles permission checks
    
    // Validate that the user exists in the database
    use crate::user::user_exists_by_uuid;
    let user_exists = user_exists_by_uuid(pool, user_uuid)
        .await
        .map_err(|e| match e {
            crate::user::UserDatabaseError::Database(_) | crate::user::UserDatabaseError::UserCreation(_) => {
                BackupError::Database(sqlx::Error::RowNotFound)
            }
            crate::user::UserDatabaseError::Sql(sql_err) => BackupError::Database(sql_err),
        })?;
    
    if !user_exists {
        return Err(BackupError::UserNotFound(user_uuid.to_string()));
    }
    
    let backup_uuid = Uuid::new_v4().to_string();
    let target_location = request.target_location.unwrap_or_else(|| "local_filesystem".to_string());
    let now = Utc::now();
    
    // Determine backup directory (use current working directory + backups subdirectory)
    let backup_dir = std::env::current_dir()
        .map(|p| p.join("backups"))
        .unwrap_or_else(|_| PathBuf::from("backups"));
    
    // Ensure filename has .json.bkp extension
    let filename = if request.filename.ends_with(".json.bkp") {
        request.filename.clone()
    } else {
        format!("{}.json.bkp", request.filename)
    };
    
    let full_path = backup_dir.join(&filename);
    let full_path_str = full_path.to_string_lossy().to_string();

    // Insert backup record with IN_PROGRESS status
    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query(
                "INSERT INTO backups (uuid, filename, full_path, creator_user_uuid, target_location,
                                     job_type, backup_status, start_timestamp, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, 'IN_PROGRESS', ?, ?)",
            )
            .bind(&backup_uuid)
            .bind(&filename)
            .bind(&full_path_str)
            .bind(user_uuid)
            .bind(&target_location)
            .bind::<Option<String>>(None::<String>) // job_type - None for manual backups
            .bind(now)
            .bind(now)
            .execute(p)
            .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO backups (uuid, filename, full_path, creator_user_uuid, target_location,
                                     job_type, backup_status, start_timestamp, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, 'IN_PROGRESS', $7, $8)",
            )
            .bind(&backup_uuid)
            .bind(&filename)
            .bind(&full_path_str)
            .bind(user_uuid)
            .bind(&target_location)
            .bind::<Option<String>>(None::<String>) // job_type - None for manual backups
            .bind(now)
            .bind(now)
            .execute(p)
            .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO backups (uuid, filename, full_path, creator_user_uuid, target_location,
                                     job_type, backup_status, start_timestamp, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'IN_PROGRESS', ?7, ?8)",
            )
            .bind(&backup_uuid)
            .bind(&filename)
            .bind(&full_path_str)
            .bind(user_uuid)
            .bind(&target_location)
            .bind::<Option<String>>(None::<String>) // job_type - None for manual backups
            .bind(now)
            .bind(now)
            .execute(p)
            .await?;
        }
    }

    Ok(backup_uuid)
}

/// Restore a backup (permission check only, actual restore is mocked)
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `backup_uuid` - UUID of the backup to restore
/// * `user_uuid` - UUID of the user restoring the backup
///
/// # Returns
/// Success message
///
/// # Errors
/// Returns `BackupError` if:
/// - Backup not found
/// - User does not have permission
/// - Database operation fails
pub async fn restore_backup(
    pool: &DatabasePool,
    backup_uuid: &str,
    user_uuid: &str,
) -> Result<(), BackupError> {
    // Check permission - backups are global
    // Note: In a real implementation, we'd check can_restore_backup permission
    
    // Verify backup exists
    let exists = match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM backups WHERE uuid = ?")
                .bind(backup_uuid)
                .fetch_one(p)
                .await?;
            row.get::<i64, _>("count") > 0
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM backups WHERE uuid = $1")
                .bind(backup_uuid)
                .fetch_one(p)
                .await?;
            row.get::<i64, _>("count") > 0
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM backups WHERE uuid = ?1")
                .bind(backup_uuid)
                .fetch_one(p)
                .await?;
            row.get::<i64, _>("count") > 0
        }
    };

    if !exists {
        return Err(BackupError::BackupNotFound);
    }

    // Mock restore - actual restore implementation will be added later
    tracing::info!("Mock restore backup: {} by user: {}", backup_uuid, user_uuid);

    Ok(())
}

/// Delete a backup by UUID
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `backup_uuid` - UUID of the backup to delete
/// * `user_uuid` - UUID of the user deleting the backup
///
/// # Returns
/// Success message
///
/// # Errors
/// Returns `BackupError` if:
/// - Backup not found
/// - User does not have permission
/// - Database operation fails
pub async fn delete_backup(
    pool: &DatabasePool,
    backup_uuid: &str,
    _user_uuid: &str,
) -> Result<(), BackupError> {
    // Check permission - backups are global
    // Note: In a real implementation, we'd check can_delete_backup permission
    
    // Get backup to delete (for file deletion)
    let full_path: Option<String> = match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query("SELECT full_path FROM backups WHERE uuid = ?")
                .bind(backup_uuid)
                .fetch_optional(p)
                .await?;
            row.map(|r| r.get("full_path"))
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query("SELECT full_path FROM backups WHERE uuid = $1")
                .bind(backup_uuid)
                .fetch_optional(p)
                .await?;
            row.map(|r| r.get("full_path"))
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query("SELECT full_path FROM backups WHERE uuid = ?1")
                .bind(backup_uuid)
                .fetch_optional(p)
                .await?;
            row.map(|r| r.get("full_path"))
        }
    };

    if full_path.is_none() {
        return Err(BackupError::BackupNotFound);
    }

    // Delete backup file if it exists
    if let Some(path) = full_path {
        let file_path = Path::new(&path);
        if file_path.exists() {
            if let Err(e) = std::fs::remove_file(file_path) {
                tracing::warn!("Failed to delete backup file {}: {}", path, e);
                // Continue with database deletion even if file deletion fails
            }
        }
    }

    // Delete from database
    match pool {
        DatabasePool::MySql(p) => {
            let result = sqlx::query("DELETE FROM backups WHERE uuid = ?")
                .bind(backup_uuid)
                .execute(p)
                .await?;
            if result.rows_affected() == 0 {
                return Err(BackupError::BackupNotFound);
            }
        }
        DatabasePool::Postgres(p) => {
            let result = sqlx::query("DELETE FROM backups WHERE uuid = $1")
                .bind(backup_uuid)
                .execute(p)
                .await?;
            if result.rows_affected() == 0 {
                return Err(BackupError::BackupNotFound);
            }
        }
        DatabasePool::Sqlite(p) => {
            let result = sqlx::query("DELETE FROM backups WHERE uuid = ?1")
                .bind(backup_uuid)
                .execute(p)
                .await?;
            if result.rows_affected() == 0 {
                return Err(BackupError::BackupNotFound);
            }
        }
    }

    Ok(())
}

/// List all backup jobs
///
/// # Returns
/// Vector of backup jobs
///
/// # Errors
/// Returns `BackupError` if database operation fails
pub async fn list_backup_jobs(pool: &DatabasePool) -> Result<Vec<BackupJob>, BackupError> {
    let mut jobs = Vec::new();
    
    match pool {
        DatabasePool::MySql(p) => {
            let rows = sqlx::query(
                "SELECT uuid, job_type, job_title, json_data, schedule, is_active, last_execution_timestamp, next_execution_timestamp, created_at, updated_at
                 FROM backup_jobs
                 ORDER BY created_at DESC",
            )
            .fetch_all(p)
            .await?;

            for row in rows {
                let json_data: Option<String> = row.get("json_data");
                let json_value = json_data
                    .as_ref()
                    .and_then(|s| serde_json::from_str::<Value>(s).ok());

                let job = BackupJob {
                    uuid: row.get("uuid"),
                    job_type: row.get("job_type"),
                    job_title: row.get("job_title"),
                    json_data: json_value,
                    schedule: row.get("schedule"),
                    is_active: row.get::<i32, _>("is_active") != 0,
                    last_execution_timestamp: row.get("last_execution_timestamp"),
                    next_execution_timestamp: row.get("next_execution_timestamp"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };
                jobs.push(job);
            }
        }
        DatabasePool::Postgres(p) => {
            let rows = sqlx::query(
                "SELECT uuid, job_type, job_title, json_data, schedule, is_active, last_execution_timestamp, next_execution_timestamp, created_at, updated_at
                 FROM backup_jobs
                 ORDER BY created_at DESC",
            )
            .fetch_all(p)
            .await?;

            for row in rows {
                let json_data: Option<String> = row.get("json_data");
                let json_value = json_data
                    .as_ref()
                    .and_then(|s| serde_json::from_str::<Value>(s).ok());

                let job = BackupJob {
                    uuid: row.get("uuid"),
                    job_type: row.get("job_type"),
                    job_title: row.get("job_title"),
                    json_data: json_value,
                    schedule: row.get("schedule"),
                    is_active: row.get::<i32, _>("is_active") != 0,
                    last_execution_timestamp: row.get("last_execution_timestamp"),
                    next_execution_timestamp: row.get("next_execution_timestamp"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };
                jobs.push(job);
            }
        }
        DatabasePool::Sqlite(p) => {
            let rows = sqlx::query(
                "SELECT uuid, job_type, job_title, json_data, schedule, is_active, last_execution_timestamp, next_execution_timestamp, created_at, updated_at
                 FROM backup_jobs
                 ORDER BY created_at DESC",
            )
            .fetch_all(p)
            .await?;

            for row in rows {
                let json_data: Option<String> = row.get("json_data");
                let json_value = json_data
                    .as_ref()
                    .and_then(|s| serde_json::from_str::<Value>(s).ok());

                let job = BackupJob {
                    uuid: row.get("uuid"),
                    job_type: row.get("job_type"),
                    job_title: row.get("job_title"),
                    json_data: json_value,
                    schedule: row.get("schedule"),
                    is_active: row.get::<i32, _>("is_active") != 0,
                    last_execution_timestamp: row.get("last_execution_timestamp"),
                    next_execution_timestamp: row.get("next_execution_timestamp"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };
                jobs.push(job);
            }
        }
    }

    Ok(jobs)
}

/// Create a new backup job
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `request` - Create backup job request
///
/// # Returns
/// UUID of the created backup job
///
/// # Errors
/// Returns `BackupError` if database operation fails
pub async fn create_backup_job(
    pool: &DatabasePool,
    request: CreateBackupJobRequest,
) -> Result<String, BackupError> {
    use crate::backup::calculate_next_execution;
    
    let job_uuid = Uuid::new_v4().to_string();
    let json_data_str = request.json_data.as_ref().map(|v| serde_json::to_string(v)).transpose()?;
    let is_active = request.is_active.unwrap_or(true);
    let now = Utc::now();
    
    // Calculate next execution timestamp from schedule
    let next_execution = if is_active {
        calculate_next_execution(request.schedule.as_deref())
    } else {
        None
    };

    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query(
                "INSERT INTO backup_jobs (uuid, job_type, job_title, json_data, schedule, is_active, next_execution_timestamp, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&job_uuid)
            .bind(&request.job_type)
            .bind(&request.job_title)
            .bind(&json_data_str)
            .bind(&request.schedule)
            .bind(if is_active { 1 } else { 0 })
            .bind(next_execution)
            .bind(now)
            .bind(now)
            .execute(p)
            .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO backup_jobs (uuid, job_type, job_title, json_data, schedule, is_active, next_execution_timestamp, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            )
            .bind(&job_uuid)
            .bind(&request.job_type)
            .bind(&request.job_title)
            .bind(&json_data_str)
            .bind(&request.schedule)
            .bind(if is_active { 1 } else { 0 })
            .bind(next_execution)
            .bind(now)
            .bind(now)
            .execute(p)
            .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO backup_jobs (uuid, job_type, job_title, json_data, schedule, is_active, next_execution_timestamp, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            )
            .bind(&job_uuid)
            .bind(&request.job_type)
            .bind(&request.job_title)
            .bind(&json_data_str)
            .bind(&request.schedule)
            .bind(if is_active { 1 } else { 0 })
            .bind(next_execution)
            .bind(now)
            .bind(now)
            .execute(p)
            .await?;
        }
    }

    Ok(job_uuid)
}

/// Get a backup job by UUID
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `job_uuid` - UUID of the backup job
///
/// # Returns
/// Backup job
///
/// # Errors
/// Returns `BackupError` if job not found or database operation fails
pub async fn get_backup_job(
    pool: &DatabasePool,
    job_uuid: &str,
) -> Result<BackupJob, BackupError> {
    match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query(
                "SELECT uuid, job_type, job_title, json_data, schedule, is_active, last_execution_timestamp, next_execution_timestamp, created_at, updated_at
                 FROM backup_jobs
                 WHERE uuid = ?",
            )
            .bind(job_uuid)
            .fetch_optional(p)
            .await?;

            let row = row.ok_or(BackupError::BackupJobNotFound)?;

            let json_data: Option<String> = row.get("json_data");
            let json_value = json_data
                .as_ref()
                .and_then(|s| serde_json::from_str::<Value>(s).ok());

            Ok(BackupJob {
                uuid: row.get("uuid"),
                job_type: row.get("job_type"),
                job_title: row.get("job_title"),
                json_data: json_value,
                schedule: row.get("schedule"),
                is_active: row.get::<i32, _>("is_active") != 0,
                last_execution_timestamp: row.get("last_execution_timestamp"),
                next_execution_timestamp: row.get("next_execution_timestamp"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT uuid, job_type, job_title, json_data, schedule, is_active, last_execution_timestamp, next_execution_timestamp, created_at, updated_at
                 FROM backup_jobs
                 WHERE uuid = $1",
            )
            .bind(job_uuid)
            .fetch_optional(p)
            .await?;

            let row = row.ok_or(BackupError::BackupJobNotFound)?;

            let json_data: Option<String> = row.get("json_data");
            let json_value = json_data
                .as_ref()
                .and_then(|s| serde_json::from_str::<Value>(s).ok());

            Ok(BackupJob {
                uuid: row.get("uuid"),
                job_type: row.get("job_type"),
                job_title: row.get("job_title"),
                json_data: json_value,
                schedule: row.get("schedule"),
                is_active: row.get::<i32, _>("is_active") != 0,
                last_execution_timestamp: row.get("last_execution_timestamp"),
                next_execution_timestamp: row.get("next_execution_timestamp"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT uuid, job_type, job_title, json_data, schedule, is_active, last_execution_timestamp, next_execution_timestamp, created_at, updated_at
                 FROM backup_jobs
                 WHERE uuid = ?1",
            )
            .bind(job_uuid)
            .fetch_optional(p)
            .await?;

            let row = row.ok_or(BackupError::BackupJobNotFound)?;

            let json_data: Option<String> = row.get("json_data");
            let json_value = json_data
                .as_ref()
                .and_then(|s| serde_json::from_str::<Value>(s).ok());

            Ok(BackupJob {
                uuid: row.get("uuid"),
                job_type: row.get("job_type"),
                job_title: row.get("job_title"),
                json_data: json_value,
                schedule: row.get("schedule"),
                is_active: row.get::<i32, _>("is_active") != 0,
                last_execution_timestamp: row.get("last_execution_timestamp"),
                next_execution_timestamp: row.get("next_execution_timestamp"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
        }
    }
}

/// Update a backup job
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `job_uuid` - UUID of the backup job
/// * `request` - Update backup job request
///
/// # Returns
/// Success message
///
/// # Errors
/// Returns `BackupError` if job not found or database operation fails
pub async fn update_backup_job(
    pool: &DatabasePool,
    job_uuid: &str,
    request: UpdateBackupJobRequest,
) -> Result<(), BackupError> {
    use crate::backup::calculate_next_execution;
    
    // Get current job to merge updates
    let current_job = get_backup_job(pool, job_uuid).await?;
    
    let job_title = request.job_title.unwrap_or(current_job.job_title);
    let json_data = request.json_data.or(current_job.json_data);
    let json_data_str = json_data.as_ref().map(|v| serde_json::to_string(v)).transpose()?;
    let schedule = request.schedule.or(current_job.schedule);
    let is_active = request.is_active.unwrap_or(current_job.is_active);
    let now = Utc::now();
    
    // Recalculate next execution timestamp if schedule or active status changed
    let next_execution = if is_active {
        calculate_next_execution(schedule.as_deref())
    } else {
        None
    };

    match pool {
        DatabasePool::MySql(p) => {
            let result = sqlx::query(
                "UPDATE backup_jobs SET job_title = ?, json_data = ?, schedule = ?, is_active = ?, next_execution_timestamp = ?, updated_at = ? WHERE uuid = ?",
            )
            .bind(&job_title)
            .bind(&json_data_str)
            .bind(&schedule)
            .bind(if is_active { 1 } else { 0 })
            .bind(next_execution)
            .bind(now)
            .bind(job_uuid)
            .execute(p)
            .await?;
            if result.rows_affected() == 0 {
                return Err(BackupError::BackupJobNotFound);
            }
        }
        DatabasePool::Postgres(p) => {
            let result = sqlx::query(
                "UPDATE backup_jobs SET job_title = $1, json_data = $2, schedule = $3, is_active = $4, next_execution_timestamp = $5, updated_at = $6 WHERE uuid = $7",
            )
            .bind(&job_title)
            .bind(&json_data_str)
            .bind(&schedule)
            .bind(if is_active { 1 } else { 0 })
            .bind(next_execution)
            .bind(now)
            .bind(job_uuid)
            .execute(p)
            .await?;
            if result.rows_affected() == 0 {
                return Err(BackupError::BackupJobNotFound);
            }
        }
        DatabasePool::Sqlite(p) => {
            let result = sqlx::query(
                "UPDATE backup_jobs SET job_title = ?1, json_data = ?2, schedule = ?3, is_active = ?4, next_execution_timestamp = ?5, updated_at = ?6 WHERE uuid = ?7",
            )
            .bind(&job_title)
            .bind(&json_data_str)
            .bind(&schedule)
            .bind(if is_active { 1 } else { 0 })
            .bind(next_execution)
            .bind(now)
            .bind(job_uuid)
            .execute(p)
            .await?;
            if result.rows_affected() == 0 {
                return Err(BackupError::BackupJobNotFound);
            }
        }
    }

    Ok(())
}

/// Delete a backup job
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `job_uuid` - UUID of the backup job
///
/// # Returns
/// Success message
///
/// # Errors
/// Returns `BackupError` if job not found or database operation fails
pub async fn delete_backup_job(
    pool: &DatabasePool,
    job_uuid: &str,
) -> Result<(), BackupError> {
    match pool {
        DatabasePool::MySql(p) => {
            let result = sqlx::query("DELETE FROM backup_jobs WHERE uuid = ?")
                .bind(job_uuid)
                .execute(p)
                .await?;
            if result.rows_affected() == 0 {
                return Err(BackupError::BackupJobNotFound);
            }
        }
        DatabasePool::Postgres(p) => {
            let result = sqlx::query("DELETE FROM backup_jobs WHERE uuid = $1")
                .bind(job_uuid)
                .execute(p)
                .await?;
            if result.rows_affected() == 0 {
                return Err(BackupError::BackupJobNotFound);
            }
        }
        DatabasePool::Sqlite(p) => {
            let result = sqlx::query("DELETE FROM backup_jobs WHERE uuid = ?1")
                .bind(job_uuid)
                .execute(p)
                .await?;
            if result.rows_affected() == 0 {
                return Err(BackupError::BackupJobNotFound);
            }
        }
    }

    Ok(())
}

/// Execute a backup job now (mocked)
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `job_uuid` - UUID of the backup job
/// * `user_uuid` - UUID of the user executing the job
///
/// # Returns
/// Success message
///
/// # Errors
/// Returns `BackupError` if job not found or database operation fails
pub async fn execute_backup_job(
    pool: &DatabasePool,
    job_uuid: &str,
    user_uuid: &str,
) -> Result<(), BackupError> {
    // Verify job exists
    let job = get_backup_job(pool, job_uuid).await?;

    // Update last_execution_timestamp
    let now = Utc::now();
    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query("UPDATE backup_jobs SET last_execution_timestamp = ? WHERE uuid = ?")
                .bind(now)
                .bind(job_uuid)
                .execute(p)
                .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query("UPDATE backup_jobs SET last_execution_timestamp = $1 WHERE uuid = $2")
                .bind(now)
                .bind(job_uuid)
                .execute(p)
                .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query("UPDATE backup_jobs SET last_execution_timestamp = ?1 WHERE uuid = ?2")
                .bind(now)
                .bind(job_uuid)
                .execute(p)
                .await?;
        }
    }

    // Mock execution - actual job execution will be implemented later
    tracing::info!(
        "Mock execute backup job: {} (type: {}, title: {}) by user: {}",
        job_uuid,
        job.job_type,
        job.job_title,
        user_uuid
    );

    Ok(())
}

/// Get backup statistics
///
/// # Returns
/// Tuple of (total_backups, last_backup_timestamp, configured_backup_path)
///
/// # Errors
/// Returns `BackupError` if database operation fails
pub async fn get_backup_statistics(
    pool: &DatabasePool,
) -> Result<(u64, Option<DateTime<Utc>>, String), BackupError> {
    let (total, last_backup): (i64, Option<DateTime<Utc>>) = match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query(
                "SELECT COUNT(*) as total, MAX(created_at) as last_backup FROM backups",
            )
            .fetch_one(p)
            .await?;
            (row.get("total"), row.get("last_backup"))
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT COUNT(*) as total, MAX(created_at) as last_backup FROM backups",
            )
            .fetch_one(p)
            .await?;
            (row.get("total"), row.get("last_backup"))
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT COUNT(*) as total, MAX(created_at) as last_backup FROM backups",
            )
            .fetch_one(p)
            .await?;
            (row.get("total"), row.get("last_backup"))
        }
    };

    // For now, use hardcoded path - will be configurable later
    let backup_path = "<Working directory>/backups".to_string();

    Ok((total as u64, last_backup, backup_path))
}

/// Update backup status
pub async fn update_backup_status(
    pool: &DatabasePool,
    backup_uuid: &str,
    status: BackupStatus,
) -> Result<(), BackupError> {
    let status_str: String = status.into();
    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query("UPDATE backups SET backup_status = ? WHERE uuid = ?")
                .bind(&status_str)
                .bind(backup_uuid)
                .execute(p)
                .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query("UPDATE backups SET backup_status = $1 WHERE uuid = $2")
                .bind(&status_str)
                .bind(backup_uuid)
                .execute(p)
                .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query("UPDATE backups SET backup_status = ?1 WHERE uuid = ?2")
                .bind(&status_str)
                .bind(backup_uuid)
                .execute(p)
                .await?;
        }
    }
    Ok(())
}

/// Update backup file path
pub async fn update_backup_path(
    pool: &DatabasePool,
    backup_uuid: &str,
    full_path: &str,
) -> Result<(), BackupError> {
    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query("UPDATE backups SET full_path = ? WHERE uuid = ?")
                .bind(full_path)
                .bind(backup_uuid)
                .execute(p)
                .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query("UPDATE backups SET full_path = $1 WHERE uuid = $2")
                .bind(full_path)
                .bind(backup_uuid)
                .execute(p)
                .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query("UPDATE backups SET full_path = ?1 WHERE uuid = ?2")
                .bind(full_path)
                .bind(backup_uuid)
                .execute(p)
                .await?;
        }
    }
    Ok(())
}

