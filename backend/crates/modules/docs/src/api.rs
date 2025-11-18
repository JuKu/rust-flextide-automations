//! Docs API endpoints
//!
//! Provides REST API endpoints for managing documentation and related resources.

use axum::{
    extract::Extension,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use flextide_core::database::DatabasePool;
use serde_json::json;

/// Create the API router for Docs endpoints
pub fn create_api_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/modules/docs/documents", get(list_documents))
}

async fn list_documents(
    Extension(_pool): Extension<DatabasePool>,
    Extension(_org_uuid): Extension<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // TODO: Implement document listing
    Ok(Json(json!({
        "documents": [],
        "total": 0
    })))
}

