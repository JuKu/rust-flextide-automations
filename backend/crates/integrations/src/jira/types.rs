//! Jira API Types

use serde::{Deserialize, Serialize};

/// Paginated response for project search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSearchResponse {
    /// Whether this is the last page of results
    #[serde(rename = "isLast")]
    pub is_last: bool,
    /// Maximum number of results per page
    #[serde(rename = "maxResults")]
    pub max_results: usize,
    /// URL for the next page of results (if available)
    #[serde(rename = "nextPage")]
    pub next_page: Option<String>,
    /// URL for this page of results
    #[serde(rename = "self")]
    pub self_url: String,
    /// Index of the first item in this page
    #[serde(rename = "startAt")]
    pub start_at: usize,
    /// Total number of projects available
    pub total: usize,
    /// List of projects in this page
    pub values: Vec<Project>,
}

/// Jira project information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Avatar URLs for different sizes
    pub avatar_urls: AvatarUrls,
    /// Project ID
    pub id: String,
    /// Insight information (if available)
    pub insight: Option<Insight>,
    /// Project key
    pub key: String,
    /// Project name
    pub name: String,
    /// Project category (if assigned)
    #[serde(rename = "projectCategory")]
    pub project_category: Option<ProjectCategory>,
    /// URL to this project
    #[serde(rename = "self")]
    pub self_url: String,
    /// Whether this is a simplified project
    pub simplified: bool,
    /// Project style (e.g., "classic", "next-gen")
    pub style: String,
}

/// Avatar URLs for different sizes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarUrls {
    /// 16x16 avatar URL
    #[serde(rename = "16x16")]
    pub size_16: String,
    /// 24x24 avatar URL
    #[serde(rename = "24x24")]
    pub size_24: String,
    /// 32x32 avatar URL
    #[serde(rename = "32x32")]
    pub size_32: String,
    /// 48x48 avatar URL
    #[serde(rename = "48x48")]
    pub size_48: String,
}

/// Insight information for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    /// Last issue update time (ISO 8601 format)
    #[serde(rename = "lastIssueUpdateTime")]
    pub last_issue_update_time: Option<String>,
    /// Total issue count
    #[serde(rename = "totalIssueCount")]
    pub total_issue_count: Option<usize>,
}

/// Project category information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCategory {
    /// Category description
    pub description: Option<String>,
    /// Category ID
    pub id: String,
    /// Category name
    pub name: String,
    /// URL to this category
    #[serde(rename = "self")]
    pub self_url: String,
}

/// Detailed project information with all expanded fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDetails {
    /// Assignee type for the project
    #[serde(rename = "assigneeType")]
    pub assignee_type: Option<String>,
    /// Avatar URLs for different sizes
    pub avatar_urls: AvatarUrls,
    /// Project components
    pub components: Option<Vec<Component>>,
    /// Project description
    pub description: Option<String>,
    /// Project email address
    pub email: Option<String>,
    /// Project ID
    pub id: String,
    /// Insight information (if available)
    pub insight: Option<Insight>,
    /// Issue types available in this project
    #[serde(rename = "issueTypes")]
    pub issue_types: Option<Vec<IssueType>>,
    /// Project key
    pub key: String,
    /// Project lead user
    pub lead: Option<User>,
    /// Project name
    pub name: String,
    /// Project category (if assigned)
    #[serde(rename = "projectCategory")]
    pub project_category: Option<ProjectCategory>,
    /// Project properties
    pub properties: Option<serde_json::Value>,
    /// Project roles
    pub roles: Option<serde_json::Value>,
    /// URL to this project
    #[serde(rename = "self")]
    pub self_url: String,
    /// Whether this is a simplified project
    pub simplified: bool,
    /// Project style (e.g., "classic", "next-gen")
    pub style: String,
    /// Project URL
    pub url: Option<String>,
    /// Project versions
    pub versions: Option<Vec<Version>>,
}

/// Jira component information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    /// Component ARI (Atlassian Resource Identifier)
    pub ari: Option<String>,
    /// Component assignee
    pub assignee: Option<User>,
    /// Assignee type
    #[serde(rename = "assigneeType")]
    pub assignee_type: Option<String>,
    /// Component description
    pub description: Option<String>,
    /// Component ID
    pub id: String,
    /// Whether assignee type is valid
    #[serde(rename = "isAssigneeTypeValid")]
    pub is_assignee_type_valid: Option<bool>,
    /// Component lead
    pub lead: Option<User>,
    /// Component metadata
    pub metadata: Option<ComponentMetadata>,
    /// Component name
    pub name: String,
    /// Project key
    pub project: Option<String>,
    /// Project ID
    #[serde(rename = "projectId")]
    pub project_id: Option<usize>,
    /// Real assignee (if different from assignee)
    #[serde(rename = "realAssignee")]
    pub real_assignee: Option<User>,
    /// Real assignee type
    #[serde(rename = "realAssigneeType")]
    pub real_assignee_type: Option<String>,
    /// URL to this component
    #[serde(rename = "self")]
    pub self_url: String,
}

/// Component metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    /// Icon URL
    pub icon: Option<String>,
}

/// Issue type information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueType {
    /// Avatar ID
    #[serde(rename = "avatarId")]
    pub avatar_id: Option<usize>,
    /// Issue type description
    pub description: Option<String>,
    /// Hierarchy level
    #[serde(rename = "hierarchyLevel")]
    pub hierarchy_level: Option<usize>,
    /// Icon URL
    #[serde(rename = "iconUrl")]
    pub icon_url: Option<String>,
    /// Issue type ID
    pub id: String,
    /// Issue type name
    pub name: String,
    /// Issue type scope (if applicable)
    pub scope: Option<IssueTypeScope>,
    /// URL to this issue type
    #[serde(rename = "self")]
    pub self_url: String,
    /// Whether this is a subtask type
    pub subtask: bool,
    /// Entity ID (for next-gen projects)
    #[serde(rename = "entityId")]
    pub entity_id: Option<String>,
}

/// Issue type scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueTypeScope {
    /// Project information
    pub project: Option<ProjectScope>,
    /// Scope type
    #[serde(rename = "type")]
    pub scope_type: Option<String>,
}

/// Project scope information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectScope {
    /// Project ID
    pub id: Option<String>,
}

/// Jira user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Account ID
    #[serde(rename = "accountId")]
    pub account_id: String,
    /// Account type
    #[serde(rename = "accountType")]
    pub account_type: Option<String>,
    /// Whether the user is active
    pub active: Option<bool>,
    /// Avatar URLs
    pub avatar_urls: Option<AvatarUrls>,
    /// Display name
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    /// User key (legacy)
    pub key: Option<String>,
    /// User name (legacy)
    pub name: Option<String>,
    /// URL to this user
    #[serde(rename = "self")]
    pub self_url: String,
}

/// Project version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    /// Whether this version is archived
    pub archived: Option<bool>,
    /// Version description
    pub description: Option<String>,
    /// Version ID
    pub id: Option<String>,
    /// Version name
    pub name: Option<String>,
    /// Whether this version is released
    pub released: Option<bool>,
    /// URL to this version
    #[serde(rename = "self")]
    pub self_url: Option<String>,
}
