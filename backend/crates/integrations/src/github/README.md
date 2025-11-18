# GitHub Integration

This module provides a client for interacting with the GitHub REST API. It supports both **GitHub.com** and **GitHub Enterprise Server**.

## Authentication

The GitHub client uses Bearer token authentication with Personal Access Tokens (PATs). You can use either:
- **Fine-grained personal access tokens** (recommended)
- **Classic personal access tokens**

You can generate tokens from your [GitHub account settings](https://github.com/settings/tokens).

### Token Types

Both fine-grained and classic personal access tokens are supported. Fine-grained tokens offer more granular permissions and better security, while classic tokens provide broader access.

## GitHub.com vs GitHub Enterprise

This client supports both GitHub.com and GitHub Enterprise Server:

- **GitHub.com**: Uses `https://api.github.com` as the base URL (default)
- **GitHub Enterprise Server**: Use `https://your-github-enterprise.com/api/v3` as the base URL

The API endpoints and authentication methods are the same for both versions.

## Usage

```rust
use integrations::github::{GitHubClient, CreateRepositoryRequest};

// Create a client with authentication
let client = GitHubClient::with_token("your-personal-access-token".to_string());

// List all organizations
let organizations = client.list_organizations(None, None).await?;

// Get a specific organization
let org = client.get_organization("github").await?;

// List repositories for an organization
let repos = client.list_organization_repositories("github", None, None, None, None).await?;

// Create a new repository (defaults to private)
let request = CreateRepositoryRequest {
    name: "my-new-repo".to_string(),
    description: Some("My awesome repository".to_string()),
    ..Default::default() // private defaults to true
};
let repo = client.create_user_repository(request).await?;
```

## Methods

### `new`

Creates a new GitHub client without authentication.

**Returns:**
- `GitHubClient` - A new client instance without authentication

**Example:**
```rust
let client = GitHubClient::new();
```

### `with_token`

Creates a new GitHub client with authentication token.

**Parameters:**
- `token: String` - Your GitHub personal access token

**Returns:**
- `GitHubClient` - A new authenticated client instance

**Example:**
```rust
let client = GitHubClient::with_token("ghp_xxxxxxxxxxxx".to_string());
```

### `with_base_url`

Creates a new GitHub client with a custom base URL (useful for GitHub Enterprise).

**Parameters:**
- `token: Option<String>` - Your GitHub personal access token (optional)
- `base_url: String` - The base URL for the GitHub API
  - For GitHub.com: `"https://api.github.com"` (default)
  - For GitHub Enterprise: `"https://your-github-enterprise.com/api/v3"`

**Returns:**
- `GitHubClient` - A new client instance

**Example:**
```rust
// GitHub Enterprise
let client = GitHubClient::with_base_url(
    Some("token".to_string()),
    "https://github.company.com/api/v3".to_string(),
);
```

### `list_organizations`

Get all organizations (paginated).

**Parameters:**
- `per_page: Option<u32>` - Number of results per page (1-100, default: 30)
- `since: Option<u64>` - Only show organizations updated after this time (optional)

**Returns:**
- `Result<Vec<OrganizationSimple>, GitHubError>` - A vector of organization objects

**Example:**
```rust
// Get all organizations with default pagination
let orgs = client.list_organizations(None, None).await?;

// Get organizations with custom page size
let orgs = client.list_organizations(Some(100), None).await?;
```

**Note:** This method automatically handles pagination and returns all organizations across all pages.

### `get_organization`

Get detailed information about a single organization.

**Parameters:**
- `org: &str` - The organization name (e.g., "github")

**Returns:**
- `Result<OrganizationFull, GitHubError>` - Complete organization details

**Example:**
```rust
let org = client.get_organization("github").await?;
println!("Organization: {}", org.name.unwrap_or(org.login));
```

### `list_user_repositories`

List repositories for the authenticated user (paginated).

**Parameters:**
- `repo_type: Option<&str>` - Filter by repository type:
  - `"all"` - All repositories the user has access to (default)
  - `"owner"` - Only repositories owned by the user (personal repos)
  - `"member"` - Only repositories where the user is a member
- `sort: Option<&str>` - Sort by: `"created"`, `"updated"`, `"pushed"`, `"full_name"` (default: `"full_name"`)
- `direction: Option<&str>` - Sort direction: `"asc"`, `"desc"` (default: `"desc"` when not using `"full_name"`, otherwise `"asc"`)
- `per_page: Option<u32>` - Number of results per page (1-100, default: 30)

**Returns:**
- `Result<Vec<Repository>, GitHubError>` - A vector of repository objects

**Example:**
```rust
// Get only personal repositories (owned by user)
let repos = client.list_user_repositories(
    Some("owner"),
    Some("updated"),
    Some("desc"),
    Some(100),
).await?;

// Get all repositories the user has access to
let repos = client.list_user_repositories(None, None, None, None).await?;
```

**Note:** This method automatically handles pagination and returns all repositories across all pages. Requires authentication.

### `list_organization_repositories`

List all repositories for an organization (paginated).

**Parameters:**
- `org: &str` - The organization name (e.g., "github")
- `repo_type: Option<&str>` - Filter by repository type:
  - `"all"` - All repositories (default)
  - `"public"` - Only public repositories
  - `"private"` - Only private repositories
  - `"forks"` - Only forked repositories
  - `"sources"` - Only source repositories
  - `"member"` - Only repositories where the user is a member
- `sort: Option<&str>` - Sort by: `"created"`, `"updated"`, `"pushed"`, `"full_name"` (default: `"full_name"`)
- `direction: Option<&str>` - Sort direction: `"asc"`, `"desc"` (default: `"asc"` when using `"full_name"`, otherwise `"desc"`)
- `per_page: Option<u32>` - Number of results per page (1-100, default: 30)

**Returns:**
- `Result<Vec<Repository>, GitHubError>` - A vector of repository objects

**Example:**
```rust
// Get all repositories for an organization
let repos = client.list_organization_repositories("github", None, None, None, None).await?;

// Get only public repositories, sorted by updated date
let repos = client.list_organization_repositories(
    "github",
    Some("public"),
    Some("updated"),
    Some("desc"),
    Some(50),
).await?;
```

**Note:** This method automatically handles pagination and returns all repositories across all pages.

### `get_repository`

Get detailed information about a single repository.

**Parameters:**
- `owner: &str` - The repository owner (user or organization name)
- `repo: &str` - The repository name

**Returns:**
- `Result<RepositoryFull, GitHubError>` - Complete repository details

**Example:**
```rust
let repo = client.get_repository("octocat", "Hello-World").await?;
println!("Repository: {}", repo.full_name);
println!("Description: {:?}", repo.description);
```

### `create_user_repository`

Create a repository for the authenticated user.

**Parameters:**
- `request: CreateRepositoryRequest` - The repository creation request

**Returns:**
- `Result<RepositoryFull, GitHubError>` - The created repository details

**Example:**
```rust
// Create a private repository (default)
let request = CreateRepositoryRequest {
    name: "my-new-repo".to_string(),
    description: Some("My awesome repository".to_string()),
    auto_init: Some(true), // Initialize with README
    ..Default::default() // private defaults to true
};
let repo = client.create_user_repository(request).await?;

// Create a public repository
let request = CreateRepositoryRequest {
    name: "public-repo".to_string(),
    description: Some("Public repository".to_string()),
    private: Some(false), // Override default to make it public
    auto_init: Some(true),
    ..Default::default()
};
let repo = client.create_user_repository(request).await?;
```

**Note:** Requires authentication. By default, repositories are created as **private** (can be overridden by setting `private: Some(false)`).

### `create_organization_repository`

Create a repository for an organization.

**Parameters:**
- `org: &str` - The organization name
- `request: CreateRepositoryRequest` - The repository creation request

**Returns:**
- `Result<RepositoryFull, GitHubError>` - The created repository details

**Example:**
```rust
let request = CreateRepositoryRequest {
    name: "org-repo".to_string(),
    description: Some("Organization repository".to_string()),
    private: Some(true), // Explicitly set to private
    auto_init: Some(true),
    has_issues: Some(true),
    has_projects: Some(true),
    ..Default::default()
};
let repo = client.create_organization_repository("myorg", request).await?;
```

**Note:** Requires authentication. The token must have appropriate permissions for the organization. By default, repositories are created as **private** (can be overridden by setting `private: Some(false)`).

## Types

### `CreateRepositoryRequest`

Request structure for creating a new repository.

**Fields:**
- `name: String` - The name of the repository (required)
- `description: Option<String>` - A short description of the repository
- `private: Option<bool>` - Whether the repository is private (default: `Some(true)`)
- `has_issues: Option<bool>` - Whether issues are enabled
- `has_projects: Option<bool>` - Whether projects are enabled
- `has_wiki: Option<bool>` - Whether the wiki is enabled
- `auto_init: Option<bool>` - Whether the repository is initialized with a README
- `gitignore_template: Option<String>` - The desired language or platform to apply to the .gitignore template
- `license_template: Option<String>` - The license keyword (e.g., "mit", "apache-2.0")
- `allow_squash_merge: Option<bool>` - Whether to allow squash merges
- `allow_merge_commit: Option<bool>` - Whether to allow merge commits
- `allow_rebase_merge: Option<bool>` - Whether to allow rebase merges
- `allow_auto_merge: Option<bool>` - Whether to allow auto-merge
- `delete_branch_on_merge: Option<bool>` - Whether to delete head branches when pull requests are merged
- `has_downloads: Option<bool>` - Whether downloads are enabled
- `is_template: Option<bool>` - Whether the repository is a template
- `default_branch: Option<String>` - The default branch name
- `team_id: Option<u64>` - The team ID to grant access to (for organization repositories)

**Default Behavior:**
- Repositories are created as **private** by default (`private: Some(true)`)
- All other fields default to `None` and use GitHub's defaults

### `OrganizationSimple`

Basic organization information.

**Fields:**
- `login: String` - Organization login name
- `id: u64` - Organization ID
- `node_id: String` - Node ID
- `url: String` - API URL
- `repos_url: String` - Repositories API URL
- `events_url: String` - Events API URL
- `hooks_url: String` - Webhooks API URL
- `issues_url: String` - Issues API URL
- `members_url: String` - Members API URL
- `public_members_url: String` - Public members API URL
- `avatar_url: String` - Avatar URL
- `description: Option<String>` - Organization description

### `OrganizationFull`

Complete organization information with all details.

**Fields:**
- All fields from `OrganizationSimple`
- `name: Option<String>` - Organization display name
- `company: Option<String>` - Company name
- `blog: Option<String>` - Blog URL
- `location: Option<String>` - Location
- `email: Option<String>` - Email address
- `twitter_username: Option<String>` - Twitter username
- `is_verified: Option<bool>` - Whether the organization is verified
- `public_repos: i64` - Number of public repositories
- `public_gists: i64` - Number of public gists
- `followers: i64` - Number of followers
- `following: i64` - Number of following
- `html_url: String` - HTML URL
- `type: String` - Type (e.g., "Organization")
- `total_private_repos: Option<i64>` - Total private repositories
- `owned_private_repos: Option<i64>` - Owned private repositories
- `plan: Option<OrganizationPlan>` - Organization plan information
- `created_at: String` - Creation timestamp
- `updated_at: String` - Last update timestamp
- And many more fields...

### `Repository`

Basic repository information.

**Fields:**
- `id: u64` - Repository ID
- `node_id: String` - Node ID
- `name: String` - Repository name
- `full_name: String` - Full repository name (owner/repo)
- `owner: User` - Repository owner
- `private: bool` - Whether the repository is private
- `html_url: String` - HTML URL
- `description: Option<String>` - Repository description
- `fork: bool` - Whether this is a fork
- `url: String` - API URL
- `forks_count: Option<u64>` - Number of forks
- `stargazers_count: Option<u64>` - Number of stars
- `watchers_count: Option<u64>` - Number of watchers
- `default_branch: Option<String>` - Default branch name
- `open_issues_count: Option<u64>` - Number of open issues
- `topics: Option<Vec<String>>` - Repository topics
- `visibility: Option<String>` - Visibility (e.g., "public", "private")
- `created_at: Option<String>` - Creation timestamp
- `updated_at: Option<String>` - Last update timestamp
- `pushed_at: Option<String>` - Last push timestamp
- And many more fields...

### `RepositoryFull`

Complete repository information with all details.

**Fields:**
- All fields from `Repository`
- `forks: Option<u64>` - Number of forks (alternative to `forks_count`)
- `watchers: Option<u64>` - Number of watchers (alternative to `watchers_count`)
- `open_issues: Option<u64>` - Number of open issues (alternative to `open_issues_count`)
- `template_repository: Option<Box<RepositoryFull>>` - Template repository (if this is a template)
- `parent: Option<Box<RepositoryFull>>` - Parent repository (if this is a fork)
- `source: Option<Box<RepositoryFull>>` - Source repository (if this is a fork)
- `organization: Option<User>` - Organization information
- `license: Option<License>` - License information
- `allow_rebase_merge: Option<bool>` - Whether rebase merges are allowed
- `allow_squash_merge: Option<bool>` - Whether squash merges are allowed
- `allow_merge_commit: Option<bool>` - Whether merge commits are allowed
- `allow_auto_merge: Option<bool>` - Whether auto-merge is allowed
- `delete_branch_on_merge: Option<bool>` - Whether to delete branch on merge
- `allow_forking: Option<bool>` - Whether forking is allowed
- `subscribers_count: Option<u64>` - Number of subscribers
- `network_count: Option<u64>` - Network count
- And many more fields...

### `User`

User/Owner information.

**Fields:**
- `login: String` - User login name
- `id: u64` - User ID
- `node_id: String` - Node ID
- `avatar_url: Option<String>` - Avatar URL
- `html_url: Option<String>` - HTML URL
- `type: String` - Type (e.g., "User", "Organization")
- `site_admin: Option<bool>` - Whether the user is a site admin
- And many more fields...

### `License`

License information.

**Fields:**
- `key: Option<String>` - License key (e.g., "mit")
- `name: Option<String>` - License name (e.g., "MIT License")
- `url: Option<String>` - License URL
- `spdx_id: Option<String>` - SPDX identifier
- `node_id: Option<String>` - Node ID
- `html_url: Option<String>` - HTML URL

## Error Handling

The client returns `GitHubError` which can be one of:
- `HttpError` - HTTP request failed (network errors, timeouts, etc.)
- `JsonError` - JSON serialization/deserialization failed
- `ApiError` - GitHub API returned an error (400 Bad Request, 500 Internal Server Error, etc.)
- `AuthenticationError` - Authentication failed (401 Unauthorized)
- `RateLimitError` - Rate limit exceeded (429 Too Many Requests or 403 with retry-after header)
- `NotFound` - Resource not found (404 Not Found)
- `InvalidRequest` - Invalid request parameters (400 Bad Request)
- `Unknown` - Unknown error

### HTTP Status Code Handling

All methods handle the following HTTP status codes:

- **200 OK** → Success, returns the requested data
- **201 Created** → Success, returns the created resource (for create methods)
- **400 Bad Request** → Returns `GitHubError::InvalidRequest` or `GitHubError::ApiError`
- **401 Unauthorized** → Returns `GitHubError::AuthenticationError`
- **403 Forbidden** → Returns `GitHubError::ApiError` or `GitHubError::RateLimitError` (if retry-after header is present)
- **404 Not Found** → Returns `GitHubError::NotFound`
- **429 Too Many Requests** → Returns `GitHubError::RateLimitError` with retry-after information
- **Other errors** (500, 503, etc.) → Returns `GitHubError::ApiError` with status code and error message

### Rate Limiting

The client automatically detects rate limit errors and includes retry-after information in the error message. When a rate limit is encountered, the error will include the number of seconds to wait before retrying.

## Pagination

Methods that return lists of items (`list_organizations`, `list_user_repositories`, `list_organization_repositories`) automatically handle pagination. They will fetch all pages and return a complete list of all items.

The pagination is handled transparently using GitHub's `Link` header. You don't need to manually manage page numbers or follow pagination links.

## API Reference

This integration uses the GitHub REST API. For more information, see:
- [GitHub REST API Documentation](https://docs.github.com/en/rest)
- [GitHub REST API Authentication](https://docs.github.com/en/rest/authentication/authenticating-to-the-rest-api)
- [GitHub REST API Organizations](https://docs.github.com/en/rest/orgs/orgs)
- [GitHub REST API Repositories](https://docs.github.com/en/rest/repos/repos)

