# Jira Integration

This module provides a client for interacting with the Jira API (v3). It supports both **Jira Cloud** and **Jira Data Center**.

## Authentication

The Jira client uses Basic Authentication with your email address and API token. You can generate an API token from your [Atlassian account settings](https://id.atlassian.com/manage-profile/security/api-tokens).

## Jira Cloud vs Data Center

This client supports both Jira Cloud and Jira Data Center:

- **Jira Cloud**: Use `https://your-domain.atlassian.net` as the base URL
- **Jira Data Center**: Use `https://your-jira-server.com` or `https://your-jira-server.com/jira` as the base URL (include the `/jira` path if your Data Center installation requires it)

The API endpoints and authentication methods are the same for both versions.

## Usage

```rust
use integrations::jira::{JiraClient, ProjectSearchResponse};

let client = JiraClient::new(
    "https://your-domain.atlassian.net".to_string(),
    "email@example.com".to_string(),
    "your-api-token".to_string(),
);

// Get all projects (first page, default max 100)
let projects = client.get_projects(None, None).await?;

// Get detailed information about a specific project
let project = client.get_project("EX").await?;
```

## Methods

### `new`

Creates a new Jira client instance.

**Parameters:**
- `base_url: String` - The base URL of your Jira instance
  - For Jira Cloud: `"https://your-domain.atlassian.net"`
  - For Jira Data Center: `"https://your-jira-server.com"` or `"https://your-jira-server.com/jira"`
- `email: String` - Your Jira account email address
- `auth_token: String` - Your Jira API token

**Returns:**
- `JiraClient` - A new client instance

**Example:**
```rust
// Jira Cloud
let client = JiraClient::new(
    "https://your-domain.atlassian.net".to_string(),
    "user@example.com".to_string(),
    "api-token".to_string(),
);

// Jira Data Center
let client = JiraClient::new(
    "https://jira.company.com/jira".to_string(),
    "user@example.com".to_string(),
    "api-token".to_string(),
);
```

### `get_projects`

Get all visible projects for the user in a paginated way.

**Parameters:**
- `start_at: Option<usize>` - The index of the first item to return (default: 0)
- `max_results: Option<usize>` - The maximum number of items to return (default: 100)

**Returns:**
- `Result<ProjectSearchResponse, JiraError>` - A paginated response containing the list of projects

**Example:**
```rust
// Get first page with default max results (100)
let response = client.get_projects(None, None).await?;

// Get second page with 50 results per page
let response = client.get_projects(Some(50), Some(50)).await?;
```

### `get_project`

Get detailed information about a single project with all expanded fields.

**Parameters:**
- `project_id_or_key: &str` - The project ID or key (e.g., "EX" or "10000")

**Returns:**
- `Result<ProjectDetails, JiraError>` - Detailed project information with all expanded fields

**Example:**
```rust
// Get project by key
let project = client.get_project("EX").await?;

// Get project by ID
let project = client.get_project("10000").await?;
```

**Note:** This method uses the `expand=*` query parameter to retrieve all available project information, including components, issue types, versions, and other expanded fields.

## Types

### `ProjectSearchResponse`

Paginated response containing project search results.

**Fields:**
- `is_last: bool` - Whether this is the last page of results
- `max_results: usize` - Maximum number of results per page
- `next_page: Option<String>` - URL for the next page of results (if available)
- `self_url: String` - URL for this page of results
- `start_at: usize` - Index of the first item in this page
- `total: usize` - Total number of projects available
- `values: Vec<Project>` - List of projects in this page

### `Project`

Represents a Jira project.

**Fields:**
- `avatar_urls: AvatarUrls` - Avatar URLs for different sizes
- `id: String` - Project ID
- `insight: Option<Insight>` - Insight information (if available)
- `key: String` - Project key
- `name: String` - Project name
- `project_category: Option<ProjectCategory>` - Project category (if assigned)
- `self_url: String` - URL to this project
- `simplified: bool` - Whether this is a simplified project
- `style: String` - Project style (e.g., "classic", "next-gen")

### `AvatarUrls`

Avatar URLs for different sizes.

**Fields:**
- `size_16: String` - 16x16 avatar URL
- `size_24: String` - 24x24 avatar URL
- `size_32: String` - 32x32 avatar URL
- `size_48: String` - 48x48 avatar URL

### `Insight`

Insight information for a project.

**Fields:**
- `last_issue_update_time: Option<String>` - Last issue update time (ISO 8601 format)
- `total_issue_count: Option<usize>` - Total issue count

### `ProjectCategory`

Project category information.

**Fields:**
- `description: Option<String>` - Category description
- `id: String` - Category ID
- `name: String` - Category name
- `self_url: String` - URL to this category

### `ProjectDetails`

Detailed project information with all expanded fields.

**Fields:**
- `assignee_type: Option<String>` - Assignee type for the project
- `avatar_urls: AvatarUrls` - Avatar URLs for different sizes
- `components: Option<Vec<Component>>` - Project components
- `description: Option<String>` - Project description
- `email: Option<String>` - Project email address
- `id: String` - Project ID
- `insight: Option<Insight>` - Insight information (if available)
- `issue_types: Option<Vec<IssueType>>` - Issue types available in this project
- `key: String` - Project key
- `lead: Option<User>` - Project lead user
- `name: String` - Project name
- `project_category: Option<ProjectCategory>` - Project category (if assigned)
- `properties: Option<serde_json::Value>` - Project properties
- `roles: Option<serde_json::Value>` - Project roles
- `self_url: String` - URL to this project
- `simplified: bool` - Whether this is a simplified project
- `style: String` - Project style (e.g., "classic", "next-gen")
- `url: Option<String>` - Project URL
- `versions: Option<Vec<Version>>` - Project versions

### `Component`

Jira component information.

**Fields:**
- `ari: Option<String>` - Component ARI (Atlassian Resource Identifier)
- `assignee: Option<User>` - Component assignee
- `assignee_type: Option<String>` - Assignee type
- `description: Option<String>` - Component description
- `id: String` - Component ID
- `is_assignee_type_valid: Option<bool>` - Whether assignee type is valid
- `lead: Option<User>` - Component lead
- `metadata: Option<ComponentMetadata>` - Component metadata
- `name: String` - Component name
- `project: Option<String>` - Project key
- `project_id: Option<usize>` - Project ID
- `real_assignee: Option<User>` - Real assignee (if different from assignee)
- `real_assignee_type: Option<String>` - Real assignee type
- `self_url: String` - URL to this component

### `ComponentMetadata`

Component metadata.

**Fields:**
- `icon: Option<String>` - Icon URL

### `IssueType`

Issue type information.

**Fields:**
- `avatar_id: Option<usize>` - Avatar ID
- `description: Option<String>` - Issue type description
- `hierarchy_level: Option<usize>` - Hierarchy level
- `icon_url: Option<String>` - Icon URL
- `id: String` - Issue type ID
- `name: String` - Issue type name
- `scope: Option<IssueTypeScope>` - Issue type scope (if applicable)
- `self_url: String` - URL to this issue type
- `subtask: bool` - Whether this is a subtask type
- `entity_id: Option<String>` - Entity ID (for next-gen projects)

### `IssueTypeScope`

Issue type scope information.

**Fields:**
- `project: Option<ProjectScope>` - Project information
- `scope_type: Option<String>` - Scope type

### `ProjectScope`

Project scope information.

**Fields:**
- `id: Option<String>` - Project ID

### `User`

Jira user information.

**Fields:**
- `account_id: String` - Account ID
- `account_type: Option<String>` - Account type
- `active: Option<bool>` - Whether the user is active
- `avatar_urls: Option<AvatarUrls>` - Avatar URLs
- `display_name: Option<String>` - Display name
- `key: Option<String>` - User key (legacy)
- `name: Option<String>` - User name (legacy)
- `self_url: String` - URL to this user

### `Version`

Project version information.

**Fields:**
- `archived: Option<bool>` - Whether this version is archived
- `description: Option<String>` - Version description
- `id: Option<String>` - Version ID
- `name: Option<String>` - Version name
- `released: Option<bool>` - Whether this version is released
- `self_url: Option<String>` - URL to this version

## Error Handling

The client returns `JiraError` which can be one of:
- `HttpError` - HTTP request failed (network errors, timeouts, etc.)
- `JsonError` - JSON serialization/deserialization failed
- `ApiError` - Jira API returned an error (404 Not Found, 500 Internal Server Error, etc.)
- `AuthenticationError` - Authentication failed (401 Unauthorized)
- `InvalidRequest` - Invalid request parameters (400 Bad Request)

### HTTP Status Code Handling

Both `get_projects` and `get_project` methods handle the following HTTP status codes:

- **400 Bad Request** → Returns `JiraError::InvalidRequest`
  - Invalid query parameters (e.g., invalid `startAt` or `maxResults` values)
  - Malformed request

- **401 Unauthorized** → Returns `JiraError::AuthenticationError`
  - Invalid credentials
  - Expired API token
  - Missing authentication

- **404 Not Found** → Returns `JiraError::ApiError`
  - Project not found (for `get_project`)
  - Resource not found (for `get_projects`)

- **Other errors** (500, 503, etc.) → Returns `JiraError::ApiError` with status code and error message

## API Reference

This integration uses the Jira Cloud REST API v3. For more information, see:
- [Jira Cloud REST API v3 Documentation](https://developer.atlassian.com/cloud/jira/platform/rest/v3/)
- [Get projects paginated endpoint](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-projects/#api-rest-api-3-project-search-get)
- [Get project endpoint](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-projects/#api-rest-api-3-project-projectidorkey-get)

