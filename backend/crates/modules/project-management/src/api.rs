//! Project Management API endpoints
//!
//! Provides REST API endpoints for managing projects, tasks, and related resources.

use axum::{
    extract::Extension,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use flextide_core::database::DatabasePool;
use serde_json::json;

/// Create the API router for Project Management endpoints
pub fn create_api_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/modules/project-management/projects", get(list_projects))
}

async fn list_projects(
    Extension(_pool): Extension<DatabasePool>,
    Extension(_org_uuid): Extension<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // TODO: Implement project listing
    Ok(Json(json!({
        "projects": [],
        "total": 0
    })))
}

