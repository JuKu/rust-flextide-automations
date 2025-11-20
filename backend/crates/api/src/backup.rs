//! Backup API endpoints
//!
//! Handles all backup-related API operations including:
//! - Backup statistics
//! - Backup CRUD operations
//! - Backup job management
//! - Backup execution

use axum::{
    extract::{Extension, Path, Query, State},
    http::{header, StatusCode},
    response::{Json, Response},
    routing::{delete, get, post},
    Router,
};
use axum::body::Body;
use axum::http::HeaderValue;
use serde::Deserialize;
use serde_json::{json, Value};
use std::str::FromStr;
use cron::Schedule;

use crate::{AppState, Claims};

#[derive(Debug, Deserialize)]
pub struct ListBackupsQuery {
    #[serde(default = "crate::default_page")]
    pub page: u32,
    #[serde(default = "crate::default_limit")]
    pub limit: u32,
}

/// Get backup statistics
///
/// GET /api/admin/backups/statistics
pub async fn get_backup_statistics(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check if user is server admin
    if !claims.is_server_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Server admin access required" })),
        ));
    }

    use flextide_core::backup::database::{get_backup_statistics, list_backup_jobs};

    let (total_backups, last_backup_timestamp, backup_path) = get_backup_statistics(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get backup statistics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to get backup statistics" })),
            )
        })?;

    // Get next planned backup job
    let jobs = list_backup_jobs(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list backup jobs: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to list backup jobs" })),
            )
        })?;

    // Find next job (for now, just get the first one or most recent)
    let next_job = jobs.first().map(|j| json!({
        "uuid": j.uuid,
        "job_type": j.job_type,
        "job_title": j.job_title,
        "last_execution_timestamp": j.last_execution_timestamp
    }));

    Ok(Json(json!({
        "total_backups": total_backups,
        "last_backup_timestamp": last_backup_timestamp,
        "backup_path": backup_path,
        "next_planned_backup": next_job
    })))
}

/// List all backups with pagination
///
/// GET /api/admin/backups
pub async fn list_backups(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListBackupsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check if user is server admin
    if !claims.is_server_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Server admin access required" })),
        ));
    }

    use flextide_core::backup::database::list_backups;

    let result = list_backups(&state.db_pool, &claims.user_uuid, query.page, query.limit)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list backups: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to list backups" })),
            )
        })?;

    Ok(Json(json!(result)))
}

/// Create a new backup
///
/// POST /api/admin/backups
pub async fn create_backup(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<flextide_core::backup::CreateBackupRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check if user is server admin
    if !claims.is_server_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Server admin access required" })),
        ));
    }

    use flextide_core::backup::database::create_backup;
    use flextide_core::backup::execute_backup;
    use flextide_core::events::{Event, EventPayload};

    let backup_uuid = create_backup(&state.db_pool, &claims.user_uuid, request)
        .await
        .map_err(|e| {
            match e {
                flextide_core::backup::BackupError::UserNotFound(uuid) => {
                    tracing::error!("User not found in database: {}", uuid);
                    (
                        StatusCode::UNAUTHORIZED,
                        Json(json!({ 
                            "error": "User not found. Please log in again.",
                            "code": "USER_NOT_FOUND"
                        })),
                    )
                }
                _ => {
                    tracing::error!("Failed to create backup: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Failed to create backup" })),
                    )
                }
            }
        })?;

    // Emit backup created event
    let event = Event::new(
        "core_backup_created",
        EventPayload::new(json!({
            "entity_type": "backup",
            "entity_id": backup_uuid,
            "data": {}
        }))
    )
    .with_user(&claims.user_uuid);

    state.event_dispatcher.emit(event).await;

    // Execute backup asynchronously
    let pool_clone = state.db_pool.clone();
    let backup_uuid_clone = backup_uuid.clone();
    let backup_path = std::env::current_dir()
        .map(|p| p.join("backups").to_string_lossy().to_string())
        .unwrap_or_else(|_| "backups".to_string());
    
    tokio::spawn(async move {
        match execute_backup(&pool_clone, &backup_uuid_clone, &backup_path).await {
            Ok(_) => {
                tracing::info!("Backup execution completed successfully: {}", backup_uuid_clone);
            }
            Err(e) => {
                tracing::error!("Backup execution failed: {} - {}", backup_uuid_clone, e);
                // Update status to FAILED
                if let Err(update_err) = flextide_core::backup::database::update_backup_status(
                    &pool_clone,
                    &backup_uuid_clone,
                    flextide_core::backup::BackupStatus::Failed,
                ).await {
                    tracing::error!("Failed to update backup status to FAILED: {}", update_err);
                }
            }
        }
    });

    Ok(Json(json!({
        "uuid": backup_uuid,
        "message": "Backup created successfully"
    })))
}

/// Delete a backup
///
/// DELETE /api/admin/backups/{uuid}
pub async fn delete_backup(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(uuid): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check if user is server admin
    if !claims.is_server_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Server admin access required" })),
        ));
    }

    use flextide_core::backup::database::delete_backup;
    use flextide_core::events::{Event, EventPayload};

    delete_backup(&state.db_pool, &uuid, &claims.user_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete backup: {}", e);
            match e {
                flextide_core::backup::BackupError::BackupNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Backup not found" })),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to delete backup" })),
                ),
            }
        })?;

    // Emit backup deleted event
    let event = Event::new(
        "core_backup_deleted",
        EventPayload::new(json!({
            "entity_type": "backup",
            "entity_id": uuid,
            "data": {}
        }))
    )
    .with_user(&claims.user_uuid);

    state.event_dispatcher.emit(event).await;

    Ok(Json(json!({ "message": "Backup deleted successfully" })))
}

/// Restore a backup
///
/// POST /api/admin/backups/{uuid}/restore
pub async fn restore_backup(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(uuid): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check if user is server admin
    if !claims.is_server_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Server admin access required" })),
        ));
    }

    use flextide_core::backup::database::restore_backup;
    use flextide_core::events::{Event, EventPayload};

    restore_backup(&state.db_pool, &uuid, &claims.user_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Failed to restore backup: {}", e);
            match e {
                flextide_core::backup::BackupError::BackupNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Backup not found" })),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to restore backup" })),
                ),
            }
        })?;

    // Emit backup restored event
    let event = Event::new(
        "core_backup_restored",
        EventPayload::new(json!({
            "entity_type": "backup",
            "entity_id": uuid,
            "data": {}
        }))
    )
    .with_user(&claims.user_uuid);

    state.event_dispatcher.emit(event).await;

    Ok(Json(json!({ "message": "Backup restore initiated successfully" })))
}

/// Download a backup file
///
/// GET /api/admin/backups/{uuid}/download
pub async fn download_backup(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(uuid): Path<String>,
) -> Result<Response, (StatusCode, Json<Value>)> {
    // Check if user is server admin
    if !claims.is_server_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Server admin access required" })),
        ));
    }

    use flextide_core::backup::get_backup_by_uuid;
    use std::fs;
    use std::path::Path;

    // Get backup information
    let backup = get_backup_by_uuid(&state.db_pool, &uuid)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get backup: {}", e);
            match e {
                flextide_core::backup::BackupError::BackupNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Backup not found" })),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to get backup" })),
                ),
            }
        })?;

    // Check if backup is completed
    if backup.backup_status != flextide_core::backup::BackupStatus::Completed {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ 
                "error": format!("Backup is not completed. Current status: {:?}", backup.backup_status),
                "status": format!("{:?}", backup.backup_status)
            })),
        ));
    }

    // Check if file exists
    let file_path = Path::new(&backup.full_path);
    if !file_path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Backup file not found on filesystem" })),
        ));
    }

    // Read file contents
    let file_contents = fs::read(&backup.full_path)
        .map_err(|e| {
            tracing::error!("Failed to read backup file {}: {}", backup.full_path, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to read backup file" })),
            )
        })?;

    // Create response with file contents
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        )
        .header(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!("attachment; filename=\"{}\"", backup.filename))
                .map_err(|e| {
                    tracing::error!("Failed to create content-disposition header: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Failed to create download response" })),
                    )
                })?,
        )
        .body(Body::from(file_contents))
        .map_err(|e| {
            tracing::error!("Failed to create response body: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to create download response" })),
            )
        })?;

    Ok(response)
}

/// List all backup jobs
///
/// GET /api/admin/backup-jobs
pub async fn list_backup_jobs(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check if user is server admin
    if !claims.is_server_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Server admin access required" })),
        ));
    }

    use flextide_core::backup::database::list_backup_jobs;

    let jobs = list_backup_jobs(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list backup jobs: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to list backup jobs" })),
            )
        })?;

    Ok(Json(json!(jobs)))
}

/// Get a backup job by UUID
///
/// GET /api/admin/backup-jobs/{uuid}
pub async fn get_backup_job(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(uuid): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check if user is server admin
    if !claims.is_server_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Server admin access required" })),
        ));
    }

    use flextide_core::backup::database::get_backup_job;

    let job = get_backup_job(&state.db_pool, &uuid)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get backup job: {}", e);
            match e {
                flextide_core::backup::BackupError::BackupJobNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Backup job not found" })),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to get backup job" })),
                ),
            }
        })?;

    Ok(Json(json!(job)))
}

/// Validate backup job request
fn validate_backup_job_request(
    job_title: &str,
    job_type: &str,
    schedule: Option<&str>,
    is_active: Option<bool>,
) -> Result<(), (StatusCode, Json<Value>)> {
    // Validate job_title
    let job_title = job_title.trim();
    if job_title.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Job title is required and cannot be empty" })),
        ));
    }
    if job_title.len() > 255 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Job title cannot exceed 255 characters" })),
        ));
    }

    // Validate job_type
    let job_type = job_type.trim();
    if job_type.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Job type is required and cannot be empty" })),
        ));
    }
    if job_type.len() > 50 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Job type cannot exceed 50 characters" })),
        ));
    }
    
    // Validate job_type is one of the allowed values
    let allowed_job_types = ["database_json_backup"];
    if !allowed_job_types.contains(&job_type) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ 
                "error": format!("Invalid job type: {}. Allowed types: {}", job_type, allowed_job_types.join(", "))
            })),
        ));
    }

    // Validate schedule if provided or if job is active
    if let Some(schedule_str) = schedule {
        let schedule_str = schedule_str.trim();
        if !schedule_str.is_empty() {
            // Validate cron expression (normalization happens in create/update endpoints)
            if let Err(e) = validate_cron_schedule(schedule_str) {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({ 
                        "error": format!("Invalid cron schedule: {}", e)
                    })),
                ));
            }
        }
    }

    // If job is active, schedule should be provided
    if is_active.unwrap_or(true) {
        if let Some(schedule_str) = schedule {
            let schedule_str = schedule_str.trim();
            if schedule_str.is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({ 
                        "error": "Schedule is required when job is active"
                    })),
                ));
            }
        } else {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ 
                    "error": "Schedule is required when job is active"
                })),
            ));
        }
    }

    Ok(())
}

/// Normalize cron schedule to 6 fields (add seconds if 5 fields provided)
/// 
/// The cron crate 0.15 expects 6 fields: second minute hour day month weekday
/// This function converts 5-field expressions (minute hour day month weekday) to 6 fields
/// by prepending "0" for seconds.
fn normalize_cron_schedule(schedule: &str) -> String {
    let schedule = schedule.trim();
    let parts: Vec<&str> = schedule.split_whitespace().collect();
    
    match parts.len() {
        5 => {
            // 5 fields: minute hour day month weekday -> prepend "0" for seconds
            format!("0 {}", schedule)
        }
        6 => {
            // Already 6 fields, return as-is
            schedule.to_string()
        }
        _ => {
            // Invalid, but return as-is (validation will catch it)
            schedule.to_string()
        }
    }
}

/// Validate cron schedule expression
/// 
/// Accepts both 5-field (minute hour day month weekday) and 6-field (second minute hour day month weekday) formats.
/// The cron crate 0.15 expects 6 fields, so 5-field expressions are normalized by prepending "0" for seconds.
fn validate_cron_schedule(schedule: &str) -> Result<String, String> {
    // Trim whitespace and ensure it's not empty
    let schedule = schedule.trim();
    if schedule.is_empty() {
        return Err("Cron schedule cannot be empty".to_string());
    }
    
    // Validate basic format: should have 5 or 6 space-separated fields
    let parts: Vec<&str> = schedule.split_whitespace().collect();
    if parts.len() != 5 && parts.len() != 6 {
        return Err(format!(
            "Cron expression must have 5 fields (minute hour day month weekday) or 6 fields (second minute hour day month weekday), found {}",
            parts.len()
        ));
    }
    
    // Normalize to 6 fields (cron crate 0.15 requires 6 fields)
    let normalized = normalize_cron_schedule(schedule);
    
    // Log the schedule for debugging
    tracing::debug!("Validating cron schedule: '{}' -> normalized: '{}'", schedule, normalized);
    
    match Schedule::from_str(&normalized) {
        Ok(_) => Ok(normalized),
        Err(e) => {
            // Log the full error for debugging
            tracing::warn!("Cron validation failed for '{}' (normalized: '{}'): {:?}", schedule, normalized, e);
            
            // The error might include a ^ character indicating parse position
            // Clean it up for better user experience
            let error_msg = format!("{}", e);
            // Remove the ^ and any surrounding whitespace/newlines
            let cleaned: String = error_msg
                .lines()
                .map(|line| line.replace("^", "").trim().to_string())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            
            // If cleaned is empty, provide a generic error
            if cleaned.is_empty() {
                Err(format!("Invalid cron expression: {}", schedule))
            } else {
                Err(cleaned)
            }
        },
    }
}

/// Create a new backup job
///
/// POST /api/admin/backup-jobs
pub async fn create_backup_job(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<flextide_core::backup::CreateBackupJobRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check if user is server admin
    if !claims.is_server_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Server admin access required" })),
        ));
    }

    // Validate request and normalize schedule
    validate_backup_job_request(
        &request.job_title,
        &request.job_type,
        request.schedule.as_deref(),
        request.is_active,
    )?;

    // Normalize schedule (convert 5 fields to 6 fields if needed)
    let mut normalized_request = request;
    if let Some(ref schedule) = normalized_request.schedule {
        if let Ok(normalized) = validate_cron_schedule(schedule) {
            normalized_request.schedule = Some(normalized);
        }
    }

    use flextide_core::backup::database::create_backup_job;
    use flextide_core::events::{Event, EventPayload};

    let job_uuid = create_backup_job(&state.db_pool, normalized_request)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create backup job: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to create backup job" })),
            )
        })?;

    // Emit backup job created event
    let event = Event::new(
        "core_backup_job_created",
        EventPayload::new(json!({
            "entity_type": "backup_job",
            "entity_id": job_uuid,
            "data": {}
        }))
    )
    .with_user(&claims.user_uuid);

    state.event_dispatcher.emit(event).await;

    Ok(Json(json!({
        "uuid": job_uuid,
        "message": "Backup job created successfully"
    })))
}

/// Update a backup job
///
/// PUT /api/admin/backup-jobs/{uuid}
pub async fn update_backup_job(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(uuid): Path<String>,
    Json(request): Json<flextide_core::backup::UpdateBackupJobRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check if user is server admin
    if !claims.is_server_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Server admin access required" })),
        ));
    }

    use flextide_core::backup::database::{get_backup_job, update_backup_job};
    use flextide_core::events::{Event, EventPayload};

    // Get current job to validate against existing values
    let current_job = get_backup_job(&state.db_pool, &uuid)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get backup job: {}", e);
            match e {
                flextide_core::backup::BackupError::BackupJobNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Backup job not found" })),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to get backup job" })),
                ),
            }
        })?;

    // Check if at least one field is being updated
    let has_updates = request.job_title.is_some()
        || request.schedule.is_some()
        || request.is_active.is_some()
        || request.json_data.is_some();
    
    if !has_updates {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "At least one field must be provided for update" })),
        ));
    }

    // Validate request using current values as fallback
    let _job_title = request.job_title.as_deref().unwrap_or(&current_job.job_title);
    let _schedule = request.schedule.as_deref().or(current_job.schedule.as_deref());
    let is_active = request.is_active.unwrap_or(current_job.is_active);
    
    // Validate job_title if provided
    if let Some(ref title) = request.job_title {
        let title = title.trim();
        if title.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Job title cannot be empty" })),
            ));
        }
        if title.len() > 255 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Job title cannot exceed 255 characters" })),
            ));
        }
    }

    // Validate schedule if provided
    if let Some(schedule_str) = request.schedule.as_deref() {
        let schedule_str = schedule_str.trim();
        if !schedule_str.is_empty() {
            if let Err(e) = validate_cron_schedule(schedule_str) {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({ 
                        "error": format!("Invalid cron schedule: {}", e)
                    })),
                ));
            }
        }
    }

    // If job will be active, schedule must be provided (either new or existing)
    if is_active {
        let final_schedule = request.schedule.as_deref().or(current_job.schedule.as_deref());
        if final_schedule.map(|s| s.trim().is_empty()).unwrap_or(true) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ 
                    "error": "Schedule is required when job is active"
                })),
            ));
        }
    }

    update_backup_job(&state.db_pool, &uuid, request)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update backup job: {}", e);
            match e {
                flextide_core::backup::BackupError::BackupJobNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Backup job not found" })),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to update backup job" })),
                ),
            }
        })?;

    // Emit backup job updated event
    let event = Event::new(
        "core_backup_job_updated",
        EventPayload::new(json!({
            "entity_type": "backup_job",
            "entity_id": uuid,
            "data": {}
        }))
    )
    .with_user(&claims.user_uuid);

    state.event_dispatcher.emit(event).await;

    Ok(Json(json!({ "message": "Backup job updated successfully" })))
}

/// Delete a backup job
///
/// DELETE /api/admin/backup-jobs/{uuid}
pub async fn delete_backup_job(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(uuid): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check if user is server admin
    if !claims.is_server_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Server admin access required" })),
        ));
    }

    use flextide_core::backup::database::delete_backup_job;
    use flextide_core::events::{Event, EventPayload};

    delete_backup_job(&state.db_pool, &uuid)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete backup job: {}", e);
            match e {
                flextide_core::backup::BackupError::BackupJobNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Backup job not found" })),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to delete backup job" })),
                ),
            }
        })?;

    // Emit backup job deleted event
    let event = Event::new(
        "core_backup_job_deleted",
        EventPayload::new(json!({
            "entity_type": "backup_job",
            "entity_id": uuid,
            "data": {}
        }))
    )
    .with_user(&claims.user_uuid);

    state.event_dispatcher.emit(event).await;

    Ok(Json(json!({ "message": "Backup job deleted successfully" })))
}

/// Execute a backup job now
///
/// POST /api/admin/backup-jobs/{uuid}/execute
pub async fn execute_backup_job(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(uuid): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check if user is server admin
    if !claims.is_server_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Server admin access required" })),
        ));
    }

    use flextide_core::backup::database::execute_backup_job;
    use flextide_core::events::{Event, EventPayload};

    execute_backup_job(&state.db_pool, &uuid, &claims.user_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute backup job: {}", e);
            match e {
                flextide_core::backup::BackupError::BackupJobNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Backup job not found" })),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to execute backup job" })),
                ),
            }
        })?;

    // Emit backup job executed event
    let event = Event::new(
        "core_backup_job_executed",
        EventPayload::new(json!({
            "entity_type": "backup_job",
            "entity_id": uuid,
            "data": {}
        }))
    )
    .with_user(&claims.user_uuid);

    state.event_dispatcher.emit(event).await;

    Ok(Json(json!({ "message": "Backup job executed successfully" })))
}

/// Create router for backup endpoints
pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/admin/backups/statistics", get(get_backup_statistics))
        .route("/admin/backups", get(list_backups).post(create_backup))
        .route("/admin/backups/{uuid}", delete(delete_backup))
        .route("/admin/backups/{uuid}/restore", post(restore_backup))
        .route("/admin/backups/{uuid}/download", get(download_backup))
        .route("/admin/backup-jobs", get(list_backup_jobs).post(create_backup_job))
        .route("/admin/backup-jobs/{uuid}", get(get_backup_job).put(update_backup_job).delete(delete_backup_job))
        .route("/admin/backup-jobs/{uuid}/execute", post(execute_backup_job))
}

