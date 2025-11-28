mod api;
mod area;
mod folder;
mod page;
mod summary;
mod tree;

pub use area::{
    AreaMemberPermissions, CreateDocsAreaRequest, DocsArea, DocsAreaDatabaseError,
    UpdateDocsAreaRequest, create_area, delete_area, load_area_by_uuid, update_area,
};
pub use folder::{
    CreateDocsFolderRequest, DocsFolder, DocsFolderDatabaseError, MoveDocsFolderRequest, UpdateDocsFolderRequest,
    create_folder, delete_folder, get_all_folders, list_folders, load_folder_by_uuid, move_folder, reorder_folder, update_folder, update_folder_name,
};
pub use page::{
    CreateDocsPageRequest, MoveDocsPageRequest, DocsPage, DocsPageDatabaseError, DocsPageVersion,
    DocsPageWithVersion, create_page, delete_page, generate_page_summary, get_all_pages, get_page_user_permissions,
    list_pages, list_page_versions, load_page_with_version, move_page, save_page_content, save_page_summary,
    update_page_properties,
};
pub use summary::{
    ClaudePageSummaryGenerator, GeminiPageSummaryGenerator, OpenAIPageSummaryGenerator,
    PageSummaryError, PageSummaryGenerator,
};
pub use tree::{
    build_area_tree, DocsAreaTree, DocsTreeError, FolderNode, PageNode, TreeNode, get_area_tree,
};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
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

