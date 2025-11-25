//! Docs Tree module
//!
//! Provides functionality for building tree structures from folders and pages.

use flextide_core::database::DatabasePool;
use serde::{Deserialize, Serialize};

use crate::folder::{get_all_folders, DocsFolder, DocsFolderDatabaseError};
use crate::page::{get_all_pages, DocsPage, DocsPageDatabaseError};

/// Tree node that can represent either a folder or a page
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TreeNode {
    #[serde(rename = "folder")]
    Folder(FolderNode),
    #[serde(rename = "page")]
    Page(PageNode),
}

/// Folder node in the tree structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderNode {
    pub folder: DocsFolder,
    pub children: Vec<TreeNode>,
}

/// Page node in the tree structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageNode {
    pub page: DocsPage,
    pub children: Vec<TreeNode>,
}

/// Complete tree structure for an area
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsAreaTree {
    pub folders: Vec<TreeNode>, // Root folders (no parent)
    pub pages: Vec<TreeNode>,   // Root pages (no folder, no parent)
}

/// Build a tree structure from folders and pages
///
/// # Arguments
/// * `folders` - All folders for the area
/// * `pages` - All pages for the area
///
/// # Returns
/// Returns a `DocsAreaTree` with hierarchical structure
pub fn build_area_tree(
    folders: Vec<DocsFolder>,
    pages: Vec<DocsPage>,
) -> DocsAreaTree {
    // Separate root items from children
    let root_folders: Vec<&DocsFolder> = folders
        .iter()
        .filter(|f| f.parent_folder_uuid.is_none())
        .collect();
    
    let root_pages: Vec<&DocsPage> = pages
        .iter()
        .filter(|p| p.folder_uuid.is_none() && p.parent_page_uuid.is_none())
        .collect();

    // Build folder tree nodes
    let folder_nodes: Vec<TreeNode> = root_folders
        .into_iter()
        .map(|folder| build_folder_node(folder, &folders, &pages))
        .collect();

    // Build page tree nodes (root pages not in folders)
    let page_nodes: Vec<TreeNode> = root_pages
        .into_iter()
        .map(|page| build_page_node(page, &pages))
        .collect();

    DocsAreaTree {
        folders: folder_nodes,
        pages: page_nodes,
    }
}

/// Build a folder node with its children
fn build_folder_node(
    folder: &DocsFolder,
    all_folders: &[DocsFolder],
    all_pages: &[DocsPage],
) -> TreeNode {
    let folder_uuid = &folder.uuid;
    
    // Find child folders
    let child_folders: Vec<&DocsFolder> = all_folders
        .iter()
        .filter(|f| f.parent_folder_uuid.as_deref() == Some(folder_uuid))
        .collect();

    // Find pages in this folder (without parent page - those are root pages in the folder)
    let child_pages: Vec<&DocsPage> = all_pages
        .iter()
        .filter(|p| p.folder_uuid.as_deref() == Some(folder_uuid) && p.parent_page_uuid.is_none())
        .collect();

    // Build child nodes
    let mut children: Vec<TreeNode> = Vec::new();

    // Add child folders
    for child_folder in child_folders {
        children.push(build_folder_node(child_folder, all_folders, all_pages));
    }

    // Add child pages
    for child_page in child_pages {
        children.push(build_page_node(child_page, all_pages));
    }

    // Sort children: folders first (by sort_order), then pages (by created_at)
    children.sort_by(|a, b| {
        match (a, b) {
            (TreeNode::Folder(fa), TreeNode::Folder(fb)) => {
                fa.folder.sort_order.cmp(&fb.folder.sort_order)
            }
            (TreeNode::Folder(_), TreeNode::Page(_)) => std::cmp::Ordering::Less,
            (TreeNode::Page(_), TreeNode::Folder(_)) => std::cmp::Ordering::Greater,
            (TreeNode::Page(pa), TreeNode::Page(pb)) => {
                pb.page.created_at.cmp(&pa.page.created_at) // Newer first
            }
        }
    });

    TreeNode::Folder(FolderNode {
        folder: folder.clone(),
        children,
    })
}

/// Build a page node with its children
fn build_page_node(page: &DocsPage, all_pages: &[DocsPage]) -> TreeNode {
    let page_uuid = &page.uuid;

    // Find child pages
    let child_pages: Vec<&DocsPage> = all_pages
        .iter()
        .filter(|p| p.parent_page_uuid.as_deref() == Some(page_uuid))
        .collect();

    // Build child nodes
    let mut children: Vec<TreeNode> = child_pages
        .into_iter()
        .map(|child_page| build_page_node(child_page, all_pages))
        .collect();

    // Sort children by created_at (newer first)
    children.sort_by(|a, b| {
        match (a, b) {
            (TreeNode::Page(pa), TreeNode::Page(pb)) => {
                pb.page.created_at.cmp(&pa.page.created_at)
            }
            _ => std::cmp::Ordering::Equal, // Shouldn't happen for page children
        }
    });

    TreeNode::Page(PageNode {
        page: page.clone(),
        children,
    })
}

/// Error type for tree building operations
#[derive(Debug, thiserror::Error)]
pub enum DocsTreeError {
    #[error("Folder database error: {0}")]
    FolderError(#[from] DocsFolderDatabaseError),
    #[error("Page database error: {0}")]
    PageError(#[from] DocsPageDatabaseError),
}

/// Build a tree structure for an area by fetching folders and pages from the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `organization_uuid` - UUID of the organization
/// * `area_uuid` - UUID of the area
///
/// # Returns
/// Returns a `DocsAreaTree` with hierarchical structure
///
/// # Errors
/// Returns `DocsTreeError` if database operations fail
pub async fn get_area_tree(
    pool: &DatabasePool,
    organization_uuid: &str,
    area_uuid: &str,
) -> Result<DocsAreaTree, DocsTreeError> {
    let folders = get_all_folders(pool, organization_uuid, area_uuid).await?;
    let pages = get_all_pages(pool, organization_uuid, area_uuid).await?;

    Ok(build_area_tree(folders, pages))
}

