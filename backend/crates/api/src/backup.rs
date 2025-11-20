//! Backup API endpoints
//!
//! Handles all backup-related API operations including:
//! - Backup statistics
//! - Backup CRUD operations
//! - Backup job management
//! - Backup execution

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

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
    use flextide_core::events::{Event, EventPayload};

    let backup_uuid = create_backup(&state.db_pool, &claims.user_uuid, request)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create backup: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to create backup" })),
            )
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

/// Download a backup (placeholder - actual file download will be implemented later)
///
/// GET /api/admin/backups/{uuid}/download
pub async fn download_backup(
    State(_state): State<AppState>,
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

    // TODO: Implement actual file download
    Ok(Json(json!({
        "message": "Download functionality will be implemented later",
        "backup_uuid": uuid
    })))
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

    use flextide_core::backup::database::create_backup_job;
    use flextide_core::events::{Event, EventPayload};

    let job_uuid = create_backup_job(&state.db_pool, request)
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

    use flextide_core::backup::database::update_backup_job;
    use flextide_core::events::{Event, EventPayload};

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

