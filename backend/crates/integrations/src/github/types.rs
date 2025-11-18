//! GitHub API Types

use serde::{Deserialize, Serialize};

/// Organization Simple - basic organization information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationSimple {
    pub login: String,
    pub id: u64,
    pub node_id: String,
    pub url: String,
    pub repos_url: String,
    pub events_url: String,
    pub hooks_url: String,
    pub issues_url: String,
    pub members_url: String,
    pub public_members_url: String,
    pub avatar_url: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// Organization Plan information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationPlan {
    pub name: String,
    pub space: i64,
    pub private_repos: i64,
    #[serde(default)]
    pub filled_seats: Option<i64>,
    #[serde(default)]
    pub seats: Option<i64>,
}

/// Organization Full - complete organization information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationFull {
    // Fields from OrganizationSimple
    pub login: String,
    pub id: u64,
    pub node_id: String,
    pub url: String,
    pub repos_url: String,
    pub events_url: String,
    pub hooks_url: String,
    pub issues_url: String,
    pub members_url: String,
    pub public_members_url: String,
    pub avatar_url: String,
    #[serde(default)]
    pub description: Option<String>,

    // Additional fields
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub company: Option<String>,
    #[serde(default)]
    pub blog: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub twitter_username: Option<String>,
    #[serde(default)]
    pub is_verified: Option<bool>,
    pub has_organization_projects: bool,
    pub has_repository_projects: bool,
    pub public_repos: i64,
    pub public_gists: i64,
    pub followers: i64,
    pub following: i64,
    pub html_url: String,
    pub r#type: String,
    #[serde(default)]
    pub total_private_repos: Option<i64>,
    #[serde(default)]
    pub owned_private_repos: Option<i64>,
    #[serde(default)]
    pub private_gists: Option<i64>,
    #[serde(default)]
    pub disk_usage: Option<i64>,
    #[serde(default)]
    pub collaborators: Option<i64>,
    #[serde(default)]
    pub billing_email: Option<String>,
    #[serde(default)]
    pub plan: Option<OrganizationPlan>,
    #[serde(default)]
    pub default_repository_permission: Option<String>,
    #[serde(default)]
    pub default_repository_branch: Option<String>,
    #[serde(default)]
    pub members_can_create_repositories: Option<bool>,
    #[serde(default)]
    pub two_factor_requirement_enabled: Option<bool>,
    #[serde(default)]
    pub members_allowed_repository_creation_type: Option<String>,
    #[serde(default)]
    pub members_can_create_public_repositories: Option<bool>,
    #[serde(default)]
    pub members_can_create_private_repositories: Option<bool>,
    #[serde(default)]
    pub members_can_create_internal_repositories: Option<bool>,
    #[serde(default)]
    pub members_can_create_pages: Option<bool>,
    #[serde(default)]
    pub members_can_create_public_pages: Option<bool>,
    #[serde(default)]
    pub members_can_create_private_pages: Option<bool>,
    #[serde(default)]
    pub members_can_delete_repositories: Option<bool>,
    #[serde(default)]
    pub members_can_change_repo_visibility: Option<bool>,
    #[serde(default)]
    pub members_can_invite_outside_collaborators: Option<bool>,
    #[serde(default)]
    pub members_can_delete_issues: Option<bool>,
    #[serde(default)]
    pub display_commenter_full_name_setting_enabled: Option<bool>,
    #[serde(default)]
    pub readers_can_create_discussions: Option<bool>,
    #[serde(default)]
    pub members_can_create_teams: Option<bool>,
    #[serde(default)]
    pub members_can_view_dependency_insights: Option<bool>,
    #[serde(default)]
    pub members_can_fork_private_repositories: Option<bool>,
    #[serde(default)]
    pub web_commit_signoff_required: Option<bool>,
    #[serde(default)]
    #[serde(rename = "advanced_security_enabled_for_new_repositories")]
    pub advanced_security_enabled_for_new_repos: Option<bool>,
    #[serde(default)]
    #[serde(rename = "dependabot_alerts_enabled_for_new_repositories")]
    pub dependabot_alerts_enabled_for_new_repos: Option<bool>,
    #[serde(default)]
    #[serde(rename = "dependabot_security_updates_enabled_for_new_repositories")]
    pub dependabot_security_updates_enabled_for_new_repos: Option<bool>,
    #[serde(default)]
    #[serde(rename = "dependency_graph_enabled_for_new_repositories")]
    pub dependency_graph_enabled_for_new_repos: Option<bool>,
    #[serde(default)]
    #[serde(rename = "secret_scanning_enabled_for_new_repositories")]
    pub secret_scanning_enabled_for_new_repos: Option<bool>,
    #[serde(default)]
    #[serde(rename = "secret_scanning_push_protection_enabled_for_new_repositories")]
    pub secret_scanning_push_protection_enabled_for_new_repos: Option<bool>,
    #[serde(default)]
    pub secret_scanning_push_protection_custom_link_enabled: Option<bool>,
    #[serde(default)]
    pub secret_scanning_push_protection_custom_link: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub archived_at: Option<String>,
    #[serde(default)]
    pub deploy_keys_enabled_for_repositories: Option<bool>,
}

/// User/Owner information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub login: String,
    pub id: u64,
    pub node_id: String,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub gravatar_id: Option<String>,
    pub url: String,
    #[serde(default)]
    pub html_url: Option<String>,
    #[serde(default)]
    pub followers_url: Option<String>,
    #[serde(default)]
    pub following_url: Option<String>,
    #[serde(default)]
    pub gists_url: Option<String>,
    #[serde(default)]
    pub starred_url: Option<String>,
    #[serde(default)]
    pub subscriptions_url: Option<String>,
    #[serde(default)]
    pub organizations_url: Option<String>,
    #[serde(default)]
    pub repos_url: Option<String>,
    #[serde(default)]
    pub events_url: Option<String>,
    #[serde(default)]
    pub received_events_url: Option<String>,
    pub r#type: String,
    #[serde(default)]
    pub site_admin: Option<bool>,
}

/// Repository permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryPermissions {
    #[serde(default)]
    pub admin: Option<bool>,
    #[serde(default)]
    pub push: Option<bool>,
    #[serde(default)]
    pub pull: Option<bool>,
}

/// Security feature status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFeatureStatus {
    pub status: String,
}

/// Security and analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAndAnalysis {
    #[serde(default)]
    pub advanced_security: Option<SecurityFeatureStatus>,
    #[serde(default)]
    pub secret_scanning: Option<SecurityFeatureStatus>,
    #[serde(default)]
    pub secret_scanning_push_protection: Option<SecurityFeatureStatus>,
    #[serde(default)]
    pub secret_scanning_non_provider_patterns: Option<SecurityFeatureStatus>,
}

/// Request to create a new repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRepositoryRequest {
    /// The name of the repository (required)
    pub name: String,
    /// A short description of the repository
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the repository is private (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private: Option<bool>,
    /// Whether issues are enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_issues: Option<bool>,
    /// Whether projects are enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_projects: Option<bool>,
    /// Whether the wiki is enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_wiki: Option<bool>,
    /// Whether the repository is initialized with a README
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_init: Option<bool>,
    /// The desired language or platform to apply to the .gitignore template
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gitignore_template: Option<String>,
    /// The license keyword (e.g., "mit", "apache-2.0")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_template: Option<String>,
    /// Whether to allow squash merges
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_squash_merge: Option<bool>,
    /// Whether to allow merge commits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_merge_commit: Option<bool>,
    /// Whether to allow rebase merges
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_rebase_merge: Option<bool>,
    /// Whether to allow auto-merge
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_auto_merge: Option<bool>,
    /// Whether to delete head branches when pull requests are merged
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_branch_on_merge: Option<bool>,
    /// Whether downloads are enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_downloads: Option<bool>,
    /// Whether the repository is a template
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_template: Option<bool>,
    /// The default branch name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_branch: Option<String>,
    /// The team ID to grant access to (for organization repositories)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u64>,
}

impl Default for CreateRepositoryRequest {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: None,
            private: Some(true), // Default to private repositories
            has_issues: None,
            has_projects: None,
            has_wiki: None,
            auto_init: None,
            gitignore_template: None,
            license_template: None,
            allow_squash_merge: None,
            allow_merge_commit: None,
            allow_rebase_merge: None,
            allow_auto_merge: None,
            delete_branch_on_merge: None,
            has_downloads: None,
            is_template: None,
            default_branch: None,
            team_id: None,
        }
    }
}

/// License information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub spdx_id: Option<String>,
    #[serde(default)]
    pub node_id: Option<String>,
    #[serde(default)]
    pub html_url: Option<String>,
}

/// Repository information (basic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: u64,
    pub node_id: String,
    pub name: String,
    pub full_name: String,
    pub owner: User,
    pub private: bool,
    pub html_url: String,
    #[serde(default)]
    pub description: Option<String>,
    pub fork: bool,
    pub url: String,
    #[serde(default)]
    pub archive_url: Option<String>,
    #[serde(default)]
    pub assignees_url: Option<String>,
    #[serde(default)]
    pub blobs_url: Option<String>,
    #[serde(default)]
    pub branches_url: Option<String>,
    #[serde(default)]
    pub collaborators_url: Option<String>,
    #[serde(default)]
    pub comments_url: Option<String>,
    #[serde(default)]
    pub commits_url: Option<String>,
    #[serde(default)]
    pub compare_url: Option<String>,
    #[serde(default)]
    pub contents_url: Option<String>,
    #[serde(default)]
    pub contributors_url: Option<String>,
    #[serde(default)]
    pub deployments_url: Option<String>,
    #[serde(default)]
    pub downloads_url: Option<String>,
    #[serde(default)]
    pub events_url: Option<String>,
    #[serde(default)]
    pub forks_url: Option<String>,
    #[serde(default)]
    pub git_commits_url: Option<String>,
    #[serde(default)]
    pub git_refs_url: Option<String>,
    #[serde(default)]
    pub git_tags_url: Option<String>,
    #[serde(default)]
    pub git_url: Option<String>,
    #[serde(default)]
    pub issue_comment_url: Option<String>,
    #[serde(default)]
    pub issue_events_url: Option<String>,
    #[serde(default)]
    pub issues_url: Option<String>,
    #[serde(default)]
    pub keys_url: Option<String>,
    #[serde(default)]
    pub labels_url: Option<String>,
    #[serde(default)]
    pub languages_url: Option<String>,
    #[serde(default)]
    pub merges_url: Option<String>,
    #[serde(default)]
    pub milestones_url: Option<String>,
    #[serde(default)]
    pub notifications_url: Option<String>,
    #[serde(default)]
    pub pulls_url: Option<String>,
    #[serde(default)]
    pub releases_url: Option<String>,
    #[serde(default)]
    pub ssh_url: Option<String>,
    #[serde(default)]
    pub stargazers_url: Option<String>,
    #[serde(default)]
    pub statuses_url: Option<String>,
    #[serde(default)]
    pub subscribers_url: Option<String>,
    #[serde(default)]
    pub subscription_url: Option<String>,
    #[serde(default)]
    pub tags_url: Option<String>,
    #[serde(default)]
    pub teams_url: Option<String>,
    #[serde(default)]
    pub trees_url: Option<String>,
    #[serde(default)]
    pub clone_url: Option<String>,
    #[serde(default)]
    pub mirror_url: Option<String>,
    #[serde(default)]
    pub hooks_url: Option<String>,
    #[serde(default)]
    pub svn_url: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub forks_count: Option<u64>,
    #[serde(default)]
    pub stargazers_count: Option<u64>,
    #[serde(default)]
    pub watchers_count: Option<u64>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub default_branch: Option<String>,
    #[serde(default)]
    pub open_issues_count: Option<u64>,
    #[serde(default)]
    pub is_template: Option<bool>,
    #[serde(default)]
    pub topics: Option<Vec<String>>,
    #[serde(default)]
    pub has_issues: Option<bool>,
    #[serde(default)]
    pub has_projects: Option<bool>,
    #[serde(default)]
    pub has_wiki: Option<bool>,
    #[serde(default)]
    pub has_pages: Option<bool>,
    #[serde(default)]
    pub has_downloads: Option<bool>,
    #[serde(default)]
    pub has_discussions: Option<bool>,
    #[serde(default)]
    pub archived: Option<bool>,
    #[serde(default)]
    pub disabled: Option<bool>,
    #[serde(default)]
    pub visibility: Option<String>,
    #[serde(default)]
    pub pushed_at: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub permissions: Option<RepositoryPermissions>,
    #[serde(default)]
    pub security_and_analysis: Option<SecurityAndAnalysis>,
}

/// Repository Full - complete repository information with all details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryFull {
    // All fields from Repository
    pub id: u64,
    pub node_id: String,
    pub name: String,
    pub full_name: String,
    pub owner: User,
    pub private: bool,
    pub html_url: String,
    #[serde(default)]
    pub description: Option<String>,
    pub fork: bool,
    pub url: String,
    #[serde(default)]
    pub archive_url: Option<String>,
    #[serde(default)]
    pub assignees_url: Option<String>,
    #[serde(default)]
    pub blobs_url: Option<String>,
    #[serde(default)]
    pub branches_url: Option<String>,
    #[serde(default)]
    pub collaborators_url: Option<String>,
    #[serde(default)]
    pub comments_url: Option<String>,
    #[serde(default)]
    pub commits_url: Option<String>,
    #[serde(default)]
    pub compare_url: Option<String>,
    #[serde(default)]
    pub contents_url: Option<String>,
    #[serde(default)]
    pub contributors_url: Option<String>,
    #[serde(default)]
    pub deployments_url: Option<String>,
    #[serde(default)]
    pub downloads_url: Option<String>,
    #[serde(default)]
    pub events_url: Option<String>,
    #[serde(default)]
    pub forks_url: Option<String>,
    #[serde(default)]
    pub git_commits_url: Option<String>,
    #[serde(default)]
    pub git_refs_url: Option<String>,
    #[serde(default)]
    pub git_tags_url: Option<String>,
    #[serde(default)]
    pub git_url: Option<String>,
    #[serde(default)]
    pub issue_comment_url: Option<String>,
    #[serde(default)]
    pub issue_events_url: Option<String>,
    #[serde(default)]
    pub issues_url: Option<String>,
    #[serde(default)]
    pub keys_url: Option<String>,
    #[serde(default)]
    pub labels_url: Option<String>,
    #[serde(default)]
    pub languages_url: Option<String>,
    #[serde(default)]
    pub merges_url: Option<String>,
    #[serde(default)]
    pub milestones_url: Option<String>,
    #[serde(default)]
    pub notifications_url: Option<String>,
    #[serde(default)]
    pub pulls_url: Option<String>,
    #[serde(default)]
    pub releases_url: Option<String>,
    #[serde(default)]
    pub ssh_url: Option<String>,
    #[serde(default)]
    pub stargazers_url: Option<String>,
    #[serde(default)]
    pub statuses_url: Option<String>,
    #[serde(default)]
    pub subscribers_url: Option<String>,
    #[serde(default)]
    pub subscription_url: Option<String>,
    #[serde(default)]
    pub tags_url: Option<String>,
    #[serde(default)]
    pub teams_url: Option<String>,
    #[serde(default)]
    pub trees_url: Option<String>,
    #[serde(default)]
    pub clone_url: Option<String>,
    #[serde(default)]
    pub mirror_url: Option<String>,
    #[serde(default)]
    pub hooks_url: Option<String>,
    #[serde(default)]
    pub svn_url: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub forks_count: Option<u64>,
    #[serde(default)]
    pub stargazers_count: Option<u64>,
    #[serde(default)]
    pub watchers_count: Option<u64>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub default_branch: Option<String>,
    #[serde(default)]
    pub open_issues_count: Option<u64>,
    #[serde(default)]
    pub is_template: Option<bool>,
    #[serde(default)]
    pub topics: Option<Vec<String>>,
    #[serde(default)]
    pub has_issues: Option<bool>,
    #[serde(default)]
    pub has_projects: Option<bool>,
    #[serde(default)]
    pub has_wiki: Option<bool>,
    #[serde(default)]
    pub has_pages: Option<bool>,
    #[serde(default)]
    pub has_downloads: Option<bool>,
    #[serde(default)]
    pub has_discussions: Option<bool>,
    #[serde(default)]
    pub archived: Option<bool>,
    #[serde(default)]
    pub disabled: Option<bool>,
    #[serde(default)]
    pub visibility: Option<String>,
    #[serde(default)]
    pub pushed_at: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub permissions: Option<RepositoryPermissions>,
    #[serde(default)]
    pub security_and_analysis: Option<SecurityAndAnalysis>,

    // Additional fields in full repository response
    #[serde(default)]
    pub forks: Option<u64>,
    #[serde(default)]
    pub watchers: Option<u64>,
    #[serde(default)]
    pub open_issues: Option<u64>,
    #[serde(default)]
    pub template_repository: Option<Box<RepositoryFull>>,
    #[serde(default)]
    pub parent: Option<Box<RepositoryFull>>,
    #[serde(default)]
    pub source: Option<Box<RepositoryFull>>,
    #[serde(default)]
    pub organization: Option<User>,
    #[serde(default)]
    pub license: Option<License>,
    #[serde(default)]
    pub allow_rebase_merge: Option<bool>,
    #[serde(default)]
    pub allow_squash_merge: Option<bool>,
    #[serde(default)]
    pub allow_auto_merge: Option<bool>,
    #[serde(default)]
    pub delete_branch_on_merge: Option<bool>,
    #[serde(default)]
    pub allow_merge_commit: Option<bool>,
    #[serde(default)]
    pub allow_forking: Option<bool>,
    #[serde(default)]
    pub temp_clone_token: Option<String>,
    #[serde(default)]
    pub subscribers_count: Option<u64>,
    #[serde(default)]
    pub network_count: Option<u64>,
}

/// Issue label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: u64,
    pub node_id: String,
    pub url: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub default: Option<bool>,
}

/// Pull request information (when issue is a PR)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub html_url: Option<String>,
    #[serde(default)]
    pub diff_url: Option<String>,
    #[serde(default)]
    pub patch_url: Option<String>,
}

/// Milestone information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub url: String,
    pub html_url: String,
    pub labels_url: String,
    pub id: u64,
    pub node_id: String,
    pub number: u64,
    pub state: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub creator: Option<User>,
    #[serde(default)]
    pub open_issues: Option<u64>,
    #[serde(default)]
    pub closed_issues: Option<u64>,
    pub created_at: String,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub closed_at: Option<String>,
    #[serde(default)]
    pub due_on: Option<String>,
}

/// GitHub Issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: u64,
    pub node_id: String,
    pub url: String,
    pub repository_url: String,
    pub labels_url: String,
    pub comments_url: String,
    pub events_url: String,
    pub html_url: String,
    pub number: u64,
    pub state: String,
    pub title: String,
    #[serde(default)]
    pub body: Option<String>,
    pub user: User,
    #[serde(default)]
    pub labels: Option<Vec<Label>>,
    #[serde(default)]
    pub assignee: Option<User>,
    #[serde(default)]
    pub assignees: Option<Vec<User>>,
    #[serde(default)]
    pub milestone: Option<Milestone>,
    #[serde(default)]
    pub locked: Option<bool>,
    #[serde(default)]
    pub active_lock_reason: Option<String>,
    #[serde(default)]
    pub comments: Option<u64>,
    #[serde(default)]
    pub pull_request: Option<PullRequest>,
    #[serde(default)]
    pub closed_at: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub closed_by: Option<User>,
    #[serde(default)]
    pub author_association: Option<String>,
    #[serde(default)]
    pub state_reason: Option<String>,
    #[serde(default)]
    pub repository: Option<Repository>,
}

