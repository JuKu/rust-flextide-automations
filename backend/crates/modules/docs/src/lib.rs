mod api;
mod area;
mod folder;
mod page;

pub use area::{
    AreaMemberPermissions, CreateDocsAreaRequest, DocsArea, DocsAreaDatabaseError,
    UpdateDocsAreaRequest, create_area, delete_area, load_area_by_uuid, update_area,
};
pub use folder::{
    CreateDocsFolderRequest, DocsFolder, DocsFolderDatabaseError, UpdateDocsFolderRequest,
    create_folder, delete_folder, list_folders, reorder_folder, update_folder_name,
};
pub use page::{
    CreateDocsPageRequest, DocsPage, DocsPageDatabaseError, DocsPageVersion,
    DocsPageWithVersion, create_page, delete_page, get_page_user_permissions,
    list_pages, load_page_with_version,
};

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

