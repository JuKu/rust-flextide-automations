mod api;

use axum::{
    extract::Extension,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use flextide_core::database::DatabasePool;
use serde_json::json;

pub fn create_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/modules/docs/health", get(health_check))
        .merge(api::create_api_router())
}

async fn health_check() -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(json!({ "status": "ok", "module": "docs" })))
}

