# GitLab REST API Research

## Overview

The GitLab REST API provides a comprehensive interface for programmatic interaction with GitLab, enabling automation of tasks, integration with external systems, and management of GitLab resources. The API is consistent across both GitLab.com (SaaS) and self-hosted GitLab instances, including the Community Edition (CE). It enables operations such as creating and managing projects, issues, merge requests, users, groups, CI/CD pipelines, and more.

The API follows RESTful principles and uses standard HTTP methods (GET, POST, PUT, DELETE) and status codes. All responses are typically returned in JSON format, with some endpoints supporting plain text.

## API Versioning

The GitLab REST API adheres to semantic versioning, with the current major version being **v4**.

### Versioning Strategy

- **Major Version**: Currently `v4`. Backward-incompatible changes result in a major version increment.
- **Minor Versions**: Not explicitly stated, allowing for a stable API endpoint.
- **New Features**: Added within the same major version without breaking changes.
- **Major Changes**: Coincide with major GitLab releases.
- **Deprecations**: Documented in the API resources.

### Experimental and Beta Features

Elements labeled as experimental or beta, and fields behind feature flags disabled by default, can be removed at any time without notice. These are excluded from the standard deprecation process.

### Enterprise Edition Considerations

For self-managed GitLab instances, downgrading from Enterprise Edition (EE) to Community Edition (CE) may cause breaking changes if EE-specific API endpoints were used.

## Base URLs

The API base URL structure depends on the GitLab instance:

- **GitLab.com:** `https://gitlab.com/api/v4/`
- **Self-Hosted GitLab:** `https://{your-gitlab-host}/api/v4/`

### Example Base URLs

```bash
# GitLab.com
https://gitlab.com/api/v4/

# Self-hosted instance
https://gitlab.example.com/api/v4/
https://git.company.com/api/v4/
```

## Authentication

Most API requests require authentication. Unauthenticated requests typically return only public data. GitLab offers several authentication methods:

### 1. Personal Access Tokens (PATs)

Personal Access Tokens are the most common method for authenticating API requests. They are tied to individual user accounts and can be scoped with specific permissions.

#### Generating a Personal Access Token

1. Navigate to GitLab Settings → Access Tokens (or User Settings → Access Tokens)
2. Click "Add new token"
3. Configure:
   - Token name
   - Expiration date (optional)
   - Scopes/permissions (see below)
4. Generate and copy the token (shown only once)

#### Personal Access Token Scopes

Common scopes include:
- `api`: Full API access
- `read_api`: Read-only API access
- `read_user`: Read user information
- `read_repository`: Read repository contents
- `write_repository`: Write to repository
- `read_registry`: Read container registry
- `write_registry`: Write to container registry

#### Using Personal Access Tokens

**Via Header (Recommended):**
```bash
curl --header "PRIVATE-TOKEN: <your_access_token>" \
     "https://gitlab.com/api/v4/user"
```

**Via Query Parameter:**
```bash
curl "https://gitlab.com/api/v4/user?private_token=<your_access_token>"
```

**cURL Example:**
```bash
curl --header "PRIVATE-TOKEN: glpat-xxxxxxxxxxxxxxxxxxxx" \
     --header "Content-Type: application/json" \
     "https://gitlab.com/api/v4/projects"
```

**Rust Example:**
```rust
use reqwest;

let client = reqwest::Client::new();
let response = client
    .get("https://gitlab.com/api/v4/user")
    .header("PRIVATE-TOKEN", "glpat-xxxxxxxxxxxxxxxxxxxx")
    .header("Content-Type", "application/json")
    .send()
    .await?;
```

**Python Example:**
```python
import requests

headers = {
    "PRIVATE-TOKEN": "glpat-xxxxxxxxxxxxxxxxxxxx",
    "Content-Type": "application/json"
}

response = requests.get("https://gitlab.com/api/v4/user", headers=headers)
print(response.json())
```

**JavaScript/Node.js Example:**
```javascript
const axios = require('axios');

axios.get('https://gitlab.com/api/v4/user', {
  headers: {
    'PRIVATE-TOKEN': 'glpat-xxxxxxxxxxxxxxxxxxxx',
    'Content-Type': 'application/json'
  }
})
.then(response => console.log(response.data))
.catch(error => console.error(error));
```

### 2. OAuth2 Tokens

OAuth2 tokens are obtained through GitLab's OAuth2 provider and are suitable for third-party applications.

#### Using OAuth2 Tokens

**Via Header:**
```bash
curl --header "Authorization: Bearer <oauth2_token>" \
     "https://gitlab.com/api/v4/user"
```

**Via Query Parameter:**
```bash
curl "https://gitlab.com/api/v4/user?access_token=<oauth2_token>"
```

**Rust Example:**
```rust
let response = client
    .get("https://gitlab.com/api/v4/user")
    .header("Authorization", format!("Bearer {}", oauth2_token))
    .send()
    .await?;
```

### 3. Project and Group Access Tokens

Project and Group Access Tokens are scoped to specific projects or groups, providing more granular access control. They are used similarly to Personal Access Tokens.

**Usage:**
```bash
curl --header "PRIVATE-TOKEN: <project_or_group_token>" \
     "https://gitlab.com/api/v4/projects/<project_id>"
```

### 4. Session Cookie

For web-based interactions, the `_gitlab_session` cookie from an authenticated web session can be used.

**Usage:**
```bash
curl --cookie "_gitlab_session=<session_cookie>" \
     "https://gitlab.com/api/v4/user"
```

### 5. GitLab CI/CD Job Token

Specific endpoints support authentication using CI/CD job tokens. These are automatically available in CI/CD jobs.

**Usage:**
```bash
curl --header "JOB-TOKEN: $CI_JOB_TOKEN" \
     "https://gitlab.com/api/v4/projects/$CI_PROJECT_ID"
```

### 6. Impersonation Tokens

Administrators can create impersonation tokens to authenticate as a specific user. This is useful for debugging and support scenarios.

### 7. Sudo Parameter

Administrators can perform API requests on behalf of another user by passing the `sudo` parameter with the target user's username or ID.

**Usage:**
```bash
curl --header "PRIVATE-TOKEN: <admin_token>" \
     --data "sudo=username" \
     "https://gitlab.com/api/v4/user"
```

### Important Notes

- **Deploy Tokens**: Cannot be used with the GitLab public API. They are only for Git operations (clone, push, pull).
- **Token Security**: Store tokens securely and never commit them to version control.
- **Token Expiration**: Some tokens can have expiration dates. Monitor and rotate tokens as needed.

## Common API Endpoints

### Projects

#### List Projects
```bash
GET /api/v4/projects
```

**cURL Example:**
```bash
curl --header "PRIVATE-TOKEN: <token>" \
     "https://gitlab.com/api/v4/projects"
```

**Rust Example:**
```rust
let response = client
    .get("https://gitlab.com/api/v4/projects")
    .header("PRIVATE-TOKEN", token)
    .send()
    .await?;
let projects: Vec<serde_json::Value> = response.json().await?;
```

#### Get Project
```bash
GET /api/v4/projects/:id
```

#### Create Project
```bash
POST /api/v4/projects
```

**cURL Example:**
```bash
curl --request POST \
     --header "PRIVATE-TOKEN: <token>" \
     --header "Content-Type: application/json" \
     --data '{"name": "my-project", "visibility": "private"}' \
     "https://gitlab.com/api/v4/projects"
```

**Rust Example:**
```rust
let project_data = serde_json::json!({
    "name": "my-project",
    "visibility": "private"
});

let response = client
    .post("https://gitlab.com/api/v4/projects")
    .header("PRIVATE-TOKEN", token)
    .json(&project_data)
    .send()
    .await?;
```

### Issues

#### List Issues
```bash
GET /api/v4/projects/:id/issues
```

**cURL Example:**
```bash
curl --header "PRIVATE-TOKEN: <token>" \
     "https://gitlab.com/api/v4/projects/123/issues"
```

#### Create Issue
```bash
POST /api/v4/projects/:id/issues
```

**cURL Example:**
```bash
curl --request POST \
     --header "PRIVATE-TOKEN: <token>" \
     --header "Content-Type: application/json" \
     --data '{"title": "New Issue", "description": "Issue description"}' \
     "https://gitlab.com/api/v4/projects/123/issues"
```

**Rust Example:**
```rust
let issue_data = serde_json::json!({
    "title": "New Issue",
    "description": "Issue description"
});

let response = client
    .post("https://gitlab.com/api/v4/projects/123/issues")
    .header("PRIVATE-TOKEN", token)
    .json(&issue_data)
    .send()
    .await?;
```

#### Update Issue
```bash
PUT /api/v4/projects/:id/issues/:issue_iid
```

#### Close Issue
```bash
PUT /api/v4/projects/:id/issues/:issue_iid
```

**cURL Example:**
```bash
curl --request PUT \
     --header "PRIVATE-TOKEN: <token>" \
     --data "state_event=close" \
     "https://gitlab.com/api/v4/projects/123/issues/1"
```

### Merge Requests

#### List Merge Requests
```bash
GET /api/v4/projects/:id/merge_requests
```

#### Create Merge Request
```bash
POST /api/v4/projects/:id/merge_requests
```

**cURL Example:**
```bash
curl --request POST \
     --header "PRIVATE-TOKEN: <token>" \
     --header "Content-Type: application/json" \
     --data '{
       "source_branch": "feature-branch",
       "target_branch": "main",
       "title": "New Feature"
     }' \
     "https://gitlab.com/api/v4/projects/123/merge_requests"
```

#### Accept Merge Request
```bash
PUT /api/v4/projects/:id/merge_requests/:merge_request_iid/merge
```

### Users

#### Get Current User
```bash
GET /api/v4/user
```

#### List Users
```bash
GET /api/v4/users
```

### Groups

#### List Groups
```bash
GET /api/v4/groups
```

#### Create Group
```bash
POST /api/v4/groups
```

### Commits

#### List Commits
```bash
GET /api/v4/projects/:id/repository/commits
```

#### Get Commit
```bash
GET /api/v4/projects/:id/repository/commits/:sha
```

### Branches

#### List Branches
```bash
GET /api/v4/projects/:id/repository/branches
```

#### Create Branch
```bash
POST /api/v4/projects/:id/repository/branches
```

### CI/CD Pipelines

#### List Pipelines
```bash
GET /api/v4/projects/:id/pipelines
```

#### Trigger Pipeline
```bash
POST /api/v4/projects/:id/pipeline
```

**cURL Example:**
```bash
curl --request POST \
     --header "PRIVATE-TOKEN: <token>" \
     --data "ref=main" \
     "https://gitlab.com/api/v4/projects/123/pipeline"
```

## Pagination

GitLab API responses are paginated to improve performance. By default, most endpoints return 20 items per page, with a maximum of 100 items per page.

### Pagination Headers

GitLab includes pagination information in response headers:

- `X-Total`: Total number of items
- `X-Total-Pages`: Total number of pages
- `X-Per-Page`: Number of items per page
- `X-Page`: Current page number
- `X-Next-Page`: Next page number (if available)
- `X-Prev-Page`: Previous page number (if available)
- `Link`: Standard HTTP Link header with pagination links

### Pagination Parameters

- `page`: Page number (default: 1)
- `per_page`: Items per page (default: 20, max: 100)

### Example: Paginated Request

**cURL Example:**
```bash
# First page
curl --header "PRIVATE-TOKEN: <token>" \
     "https://gitlab.com/api/v4/projects?page=1&per_page=50"

# Second page
curl --header "PRIVATE-TOKEN: <token>" \
     "https://gitlab.com/api/v4/projects?page=2&per_page=50"
```

**Rust Example:**
```rust
fn fetch_all_projects(client: &reqwest::Client, token: &str) -> Result<Vec<serde_json::Value>> {
    let mut all_projects = Vec::new();
    let mut page = 1;
    let per_page = 100;

    loop {
        let url = format!(
            "https://gitlab.com/api/v4/projects?page={}&per_page={}",
            page, per_page
        );
        
        let response = client
            .get(&url)
            .header("PRIVATE-TOKEN", token)
            .send()
            .await?;
        
        let projects: Vec<serde_json::Value> = response.json().await?;
        
        if projects.is_empty() {
            break;
        }
        
        all_projects.extend(projects);
        page += 1;
    }
    
    Ok(all_projects)
}
```

### Using Link Header

The `Link` header follows the RFC 5988 standard and includes `rel="next"`, `rel="prev"`, `rel="first"`, and `rel="last"` links.

## Rate Limits

GitLab enforces rate limits on API requests to prevent abuse and ensure service stability. Rate limits vary depending on the GitLab instance and user role.

### GitLab.com Rate Limits

For GitLab.com, rate limits are documented separately and may vary by subscription tier:
- Free tier: Typically lower limits
- Premium/Ultimate tiers: Higher limits

### Self-Hosted Rate Limits

Self-hosted GitLab instances can configure their own rate limits. Administrators can adjust these settings in the GitLab configuration.

### Rate Limit Headers

GitLab includes rate limit information in response headers:

- `RateLimit-Limit`: Maximum number of requests allowed
- `RateLimit-Remaining`: Number of requests remaining
- `RateLimit-Reset`: Timestamp when the rate limit resets

### Handling Rate Limits

When rate limited, GitLab returns HTTP status code `429 Too Many Requests`. Implement exponential backoff and retry logic:

**Rust Example:**
```rust
use std::time::Duration;
use tokio::time::sleep;

async fn make_request_with_retry(
    client: &reqwest::Client,
    url: &str,
    token: &str,
) -> Result<reqwest::Response> {
    let mut retries = 0;
    let max_retries = 5;
    
    loop {
        let response = client
            .get(url)
            .header("PRIVATE-TOKEN", token)
            .send()
            .await?;
        
        if response.status() == 429 {
            if retries >= max_retries {
                return Err("Rate limit exceeded, max retries reached".into());
            }
            
            // Get retry-after header or use exponential backoff
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(2_u64.pow(retries));
            
            sleep(Duration::from_secs(retry_after)).await;
            retries += 1;
            continue;
        }
        
        return Ok(response);
    }
}
```

## Error Handling

GitLab API uses standard HTTP status codes to indicate request outcomes.

### Common HTTP Status Codes

- `200 OK`: Request succeeded
- `201 Created`: Resource created successfully
- `204 No Content`: Request succeeded, no content returned
- `400 Bad Request`: Invalid request parameters
- `401 Unauthorized`: Authentication required or invalid
- `403 Forbidden`: Insufficient permissions
- `404 Not Found`: Resource not found
- `409 Conflict`: Resource conflict (e.g., duplicate name)
- `422 Unprocessable Entity`: Validation error
- `429 Too Many Requests`: Rate limit exceeded
- `500 Internal Server Error`: Server error
- `502 Bad Gateway`: Gateway error
- `503 Service Unavailable`: Service temporarily unavailable

### Error Response Format

Error responses are returned in JSON format:

```json
{
  "message": "400 Bad Request"
}
```

Or with more details:

```json
{
  "message": {
    "property": ["error message"]
  }
}
```

### Example Error Handling

**Rust Example:**
```rust
async fn handle_api_response(response: reqwest::Response) -> Result<serde_json::Value> {
    let status = response.status();
    
    if status.is_success() {
        Ok(response.json().await?)
    } else {
        let error_text = response.text().await?;
        let error: serde_json::Value = serde_json::from_str(&error_text)
            .unwrap_or_else(|_| serde_json::json!({"message": error_text}));
        
        Err(format!("API error {}: {}", status, error).into())
    }
}
```

## GitLab.com vs Self-Hosted Community Edition

### API Compatibility

The REST API is consistent across GitLab.com and self-hosted instances, including Community Edition. The same API endpoints and authentication methods work on both platforms.

### Feature Differences

While the API structure is the same, some features may differ:

- **Enterprise Edition Features**: Some endpoints may only be available in Enterprise Edition (EE), not in Community Edition (CE)
- **Rate Limits**: GitLab.com has predefined rate limits, while self-hosted instances can configure their own
- **Authentication Methods**: All authentication methods work on both platforms
- **API Versioning**: Both use the same API version (v4)

### Downgrading Considerations

When downgrading from Enterprise Edition to Community Edition, API endpoints that are EE-specific will no longer be available, which may cause breaking changes in applications that rely on them.

### Self-Hosted Configuration

Self-hosted GitLab instances can:
- Configure custom rate limits
- Enable/disable specific features
- Customize authentication methods
- Set up custom domain and SSL

## Interactive API Documentation

GitLab provides an interactive API documentation tool based on the OpenAPI specification. This tool allows users to:

- Explore API endpoints with descriptions, parameters, and example responses
- Test API calls directly within the browser
- Authenticate using a personal access token
- View request and response examples

### Accessing Interactive Documentation

1. Navigate to: https://docs.gitlab.com/api/openapi/openapi_interactive/
2. Click "Authorize" and enter your personal access token
3. Expand an endpoint to view details
4. Click "Try it out" to test the endpoint
5. Input parameters and execute the request

This feature is particularly useful for understanding API capabilities and testing integrations without external tools.

## Best Practices

### 1. Use Headers for Authentication

Prefer using headers (`PRIVATE-TOKEN` or `Authorization`) over query parameters for authentication, as headers are less likely to be logged or exposed.

### 2. Implement Pagination

Always handle pagination when fetching lists of resources. Don't assume all data fits in a single page.

### 3. Handle Rate Limits

Implement exponential backoff and retry logic for rate-limited requests. Monitor rate limit headers to optimize request patterns.

### 4. Use Appropriate Scopes

Grant only the minimum required scopes to Personal Access Tokens. Use project or group tokens when possible for better security.

### 5. Cache When Appropriate

Cache responses for resources that don't change frequently to reduce API calls and improve performance.

### 6. Error Handling

Always check HTTP status codes and handle errors appropriately. Provide meaningful error messages to users.

### 7. Use Project/Group IDs Consistently

GitLab accepts both numeric IDs and paths (e.g., `group/project`) for project identification. Be consistent in your usage.

### 8. Monitor API Usage

Track API usage to stay within rate limits and optimize request patterns.

## Additional Resources

- **GitLab REST API Documentation**: https://docs.gitlab.com/api/rest/
- **REST API Resources**: https://docs.gitlab.com/api/api_resources/
- **Interactive API Documentation**: https://docs.gitlab.com/api/openapi/openapi_interactive/
- **GitLab CLI (`glab`)**: https://docs.gitlab.com/cli/api/
- **Personal Access Tokens**: https://docs.gitlab.com/ee/user/profile/personal_access_tokens.html
- **OAuth2 Provider**: https://docs.gitlab.com/ee/api/oauth2.html
- **Rate Limits**: https://docs.gitlab.com/ee/user/gitlab_com/#rate-limits
- **Python GitLab Library**: https://python-gitlab.readthedocs.io/

## References

1. GitLab REST API Documentation: https://docs.gitlab.com/api/rest/
2. GitLab API Resources: https://docs.gitlab.com/api/api_resources/
3. GitLab Authentication: https://docs.gitlab.com/ee/api/#authentication
4. GitLab Personal Access Tokens: https://docs.gitlab.com/ee/user/profile/personal_access_tokens.html
5. GitLab OAuth2 Provider: https://docs.gitlab.com/ee/api/oauth2.html
6. GitLab Rate Limits: https://docs.gitlab.com/ee/user/gitlab_com/#rate-limits
7. GitLab Interactive API Documentation: https://docs.gitlab.com/api/openapi/openapi_interactive/
8. GitLab CLI Documentation: https://docs.gitlab.com/cli/api/
9. RFC 5988 - Web Linking: https://tools.ietf.org/html/rfc5988
10. OpenAPI Specification: https://swagger.io/specification/

