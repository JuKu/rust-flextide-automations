# GitHub REST API Research

## Overview

The GitHub REST API provides a comprehensive interface for interacting with GitHub's features programmatically. It enables developers to create, read, update, and delete repositories, issues, pull requests, and other GitHub resources. The API follows RESTful principles and uses standard HTTP methods and status codes.

## API Versioning

GitHub uses calendar-based versioning for the REST API. As of the latest version, **2022-11-28** is the current API version.

### Specifying API Version

To use a specific API version, include the `X-GitHub-Api-Version` header in your requests:

```http
X-GitHub-Api-Version: 2022-11-28
```

**Base URL:**
- **GitHub.com:** `https://api.github.com`
- **GitHub Enterprise Server:** `https://{hostname}/api/v3`

### API Version Benefits

- **Backward Compatibility:** Older API versions remain available
- **Clear Migration Path:** Gradual adoption of new features
- **Stable Interface:** Predictable behavior across versions
- **Breaking Changes:** Documented and versioned separately

## Authentication

GitHub requires authentication for most API operations. As of November 13, 2020, GitHub no longer accepts passwords for API authentication. All authenticated operations must use token-based authentication.

### 1. Personal Access Tokens (PATs)

Personal Access Tokens act as substitutes for passwords and can be generated in your GitHub account settings.

#### Classic Personal Access Tokens

**Generating a Classic PAT:**
1. Navigate to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
2. Click "Generate new token" → "Generate new token (classic)"
3. Select scopes/permissions
4. Generate and copy the token (shown only once)

**Scopes:**
- `repo`: Full control of private repositories
- `read:org`: Read organization membership
- `write:org`: Write organization membership
- `admin:org`: Full control of organization
- `gist`: Create, update, and delete gists
- `notifications`: Access notifications
- `user`: Read and update user data

**Using Classic PAT:**

**cURL Example:**
```bash
curl -H "Authorization: token YOUR_TOKEN" \
     -H "Accept: application/vnd.github+json" \
     -H "X-GitHub-Api-Version: 2022-11-28" \
     https://api.github.com/user
```

**Rust Example:**
```rust
use reqwest;

let client = reqwest::Client::new();
let response = client
    .get("https://api.github.com/user")
    .header("Authorization", format!("token {}", token))
    .header("Accept", "application/vnd.github+json")
    .header("X-GitHub-Api-Version", "2022-11-28")
    .send()
    .await?;
```

#### Fine-Grained Personal Access Tokens

Fine-grained PATs provide more granular permissions and are recommended for new integrations.

**Features:**
- Scoped to specific repositories or organizations
- Expiration dates (up to 1 year)
- More granular permissions
- Better security model

**Generating a Fine-Grained PAT:**
1. Navigate to GitHub Settings → Developer settings → Personal access tokens → Fine-grained tokens
2. Click "Generate new token"
3. Configure:
   - Token name
   - Expiration
   - Repository access (all repos, selected repos, or no access)
   - Permissions (read/write for specific resources)

**Using Fine-Grained PAT:**

Same as classic PAT, but with more restricted permissions.

### 2. GitHub Apps

GitHub Apps provide a more robust authentication method for integrations that require broader access or need to act on behalf of multiple users.

**Benefits:**
- Installation-based access
- Granular permissions
- Webhook support
- Better for production applications

**Authentication Flow:**
1. Create a GitHub App
2. Generate a private key
3. Create an installation access token
4. Use the installation token for API requests

**Rust Example:**
```rust
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

fn create_jwt(app_id: &str, private_key: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let claims = json!({
        "iat": now - 60,
        "exp": now + (10 * 60),
        "iss": app_id
    });
    
    let header = Header::new(Algorithm::RS256);
    let key = EncodingKey::from_rsa_pem(private_key.as_bytes()).unwrap();
    
    encode(&header, &claims, &key).unwrap()
}

async fn get_installation_token(
    app_id: &str,
    installation_id: &str,
    private_key: &str
) -> Result<String, Box<dyn std::error::Error>> {
    let jwt = create_jwt(app_id, private_key);
    
    let client = reqwest::Client::new();
    let response = client
        .post(&format!(
            "https://api.github.com/app/installations/{}/access_tokens",
            installation_id
        ))
        .header("Authorization", format!("Bearer {}", jwt))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?;
    
    let token_data: serde_json::Value = response.json().await?;
    Ok(token_data["token"].as_str().unwrap().to_string())
}
```

### 3. OAuth Apps

OAuth Apps use the standard OAuth 2.0 flow for user authorization.

**OAuth Flow:**
1. Redirect user to GitHub authorization URL
2. User authorizes the application
3. GitHub redirects back with authorization code
4. Exchange code for access token
5. Use access token for API requests

**Authorization URL:**
```
https://github.com/login/oauth/authorize?client_id=YOUR_CLIENT_ID&redirect_uri=YOUR_REDIRECT_URI&scope=repo
```

**Token Exchange:**
```bash
curl -X POST https://github.com/login/oauth/access_token \
  -H "Accept: application/json" \
  -d "client_id=YOUR_CLIENT_ID" \
  -d "client_secret=YOUR_CLIENT_SECRET" \
  -d "code=AUTHORIZATION_CODE"
```

## Request Headers

### Required Headers

**Accept Header:**
```http
Accept: application/vnd.github+json
```

**API Version Header:**
```http
X-GitHub-Api-Version: 2022-11-28
```

**Authorization Header:**
```http
Authorization: token YOUR_TOKEN
# or
Authorization: Bearer YOUR_TOKEN
```

### Optional Headers

**User-Agent:**
```http
User-Agent: MyApp/1.0
```

**Content-Type:**
```http
Content-Type: application/json
```

## Rate Limits

GitHub enforces rate limits to ensure fair usage of the API.

### Rate Limit Types

1. **Primary Rate Limit (Core):**
   - **Authenticated requests:** 5,000 requests per hour
   - **Unauthenticated requests:** 60 requests per hour
   - **Applies to:** Most API endpoints

2. **Search API Rate Limit:**
   - **Authenticated requests:** 30 requests per minute
   - **Unauthenticated requests:** 10 requests per minute

3. **GraphQL API Rate Limit:**
   - **5,000 points per hour**
   - Points vary by query complexity

### Rate Limit Headers

Every API response includes rate limit information:

```http
X-RateLimit-Limit: 5000
X-RateLimit-Remaining: 4999
X-RateLimit-Used: 1
X-RateLimit-Reset: 1640995200
```

**Header Fields:**
- `X-RateLimit-Limit`: Maximum requests allowed
- `X-RateLimit-Remaining`: Requests remaining in current window
- `X-RateLimit-Used`: Requests used in current window
- `X-RateLimit-Reset`: Unix timestamp when limit resets
- `X-RateLimit-Resource`: Resource type (e.g., "core", "search")

### Handling Rate Limits

**Check Rate Limit Status:**
```rust
use reqwest;

async fn check_rate_limit(token: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/rate_limit")
        .header("Authorization", format!("token {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?;
    
    let rate_limit: serde_json::Value = response.json().await?;
    let remaining = rate_limit["resources"]["core"]["remaining"].as_u64().unwrap();
    
    if remaining < 100 {
        println!("Warning: Only {} requests remaining", remaining);
    }
    
    Ok(())
}
```

**Rate Limit Exceeded Response:**
```json
{
  "message": "API rate limit exceeded for YOUR_IP. (But here's the good news: Authenticated requests get a higher rate limit. Check out the documentation for more details.)",
  "documentation_url": "https://docs.github.com/rest/overview/resources-in-the-rest-api#rate-limiting"
}
```

**HTTP Status:** `403 Forbidden`

**Best Practices:**
- Always check rate limit headers
- Implement exponential backoff on 403 responses
- Use conditional requests with ETags to reduce API calls
- Cache responses when appropriate
- Use webhooks instead of polling when possible

## Pagination

GitHub uses pagination for endpoints that return multiple items. By default, most endpoints return 30 items per page, with a maximum of 100 items per page.

### Pagination Methods

#### 1. Link Header Pagination (Recommended)

GitHub includes `Link` headers in responses with pagination information:

```http
Link: <https://api.github.com/user/repos?page=2>; rel="next",
      <https://api.github.com/user/repos?page=3>; rel="last"
```

**Link Relations:**
- `first`: First page
- `prev`: Previous page
- `next`: Next page
- `last`: Last page

**Parsing Link Headers (Rust):**
```rust
use reqwest;

fn parse_link_header(link_header: &str) -> Option<&str> {
    for link in link_header.split(',') {
        let parts: Vec<&str> = link.split(';').collect();
        if parts.len() == 2 && parts[1].contains("rel=\"next\"") {
            let url = parts[0].trim();
            return Some(url.trim_matches(|c| c == '<' || c == '>'));
        }
    }
    None
}

async fn get_all_repos(token: &str) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut all_repos = Vec::new();
    let mut url = Some("https://api.github.com/user/repos?per_page=100".to_string());
    
    while let Some(current_url) = url {
        let response = client
            .get(&current_url)
            .header("Authorization", format!("token {}", token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await?;
        
        let repos: Vec<serde_json::Value> = response.json().await?;
        all_repos.extend(repos);
        
        // Get next page URL from Link header
        if let Some(link_header) = response.headers().get("link") {
            url = parse_link_header(link_header.to_str()?);
        } else {
            url = None;
        }
    }
    
    Ok(all_repos)
}
```

#### 2. Query Parameter Pagination

You can also use query parameters directly:

**Parameters:**
- `page`: Page number (default: 1)
- `per_page`: Items per page (default: 30, max: 100)

**Example:**
```bash
curl -H "Authorization: token YOUR_TOKEN" \
     -H "Accept: application/vnd.github+json" \
     -H "X-GitHub-Api-Version: 2022-11-28" \
     "https://api.github.com/user/repos?page=2&per_page=100"
```

## Common Endpoints

### Repositories

#### List User Repositories
**Endpoint:** `GET /user/repos`

**Parameters:**
- `type`: `all`, `owner`, `member` (default: `all`)
- `sort`: `created`, `updated`, `pushed`, `full_name` (default: `full_name`)
- `direction`: `asc`, `desc` (default: `desc`)
- `per_page`: 1-100 (default: 30)
- `page`: Page number

**Example:**
```rust
async fn list_user_repos(token: &str) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/user/repos")
        .header("Authorization", format!("token {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .query(&[("type", "owner"), ("sort", "updated"), ("per_page", "100")])
        .send()
        .await?;
    
    Ok(response.json().await?)
}
```

#### Get Repository
**Endpoint:** `GET /repos/{owner}/{repo}`

**Example:**
```rust
async fn get_repo(owner: &str, repo: &str, token: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("https://api.github.com/repos/{}/{}", owner, repo))
        .header("Authorization", format!("token {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?;
    
    Ok(response.json().await?)
}
```

#### Create Repository
**Endpoint:** `POST /user/repos`

**Request Body:**
```json
{
  "name": "my-new-repo",
  "description": "Repository description",
  "private": false,
  "has_issues": true,
  "has_projects": true,
  "has_wiki": true,
  "auto_init": true
}
```

**Example:**
```rust
async fn create_repo(
    name: &str,
    description: &str,
    private: bool,
    token: &str
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.github.com/user/repos")
        .header("Authorization", format!("token {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .json(&serde_json::json!({
            "name": name,
            "description": description,
            "private": private
        }))
        .send()
        .await?;
    
    Ok(response.json().await?)
}
```

### Issues

#### List Repository Issues
**Endpoint:** `GET /repos/{owner}/{repo}/issues`

**Parameters:**
- `state`: `open`, `closed`, `all` (default: `open`)
- `labels`: Comma-separated label names
- `sort`: `created`, `updated`, `comments` (default: `created`)
- `direction`: `asc`, `desc` (default: `desc`)
- `since`: Only show issues updated after this time (ISO 8601)

**Example:**
```rust
async fn list_issues(
    owner: &str,
    repo: &str,
    state: &str,
    token: &str
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("https://api.github.com/repos/{}/{}/issues", owner, repo))
        .header("Authorization", format!("token {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .query(&[("state", state), ("per_page", "100")])
        .send()
        .await?;
    
    Ok(response.json().await?)
}
```

#### Create Issue
**Endpoint:** `POST /repos/{owner}/{repo}/issues`

**Request Body:**
```json
{
  "title": "Found a bug",
  "body": "I'm having a problem with this.",
  "labels": ["bug", "urgent"],
  "assignees": ["octocat"]
}
```

**Example:**
```rust
async fn create_issue(
    owner: &str,
    repo: &str,
    title: &str,
    body: &str,
    labels: Vec<&str>,
    token: &str
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("https://api.github.com/repos/{}/{}/issues", owner, repo))
        .header("Authorization", format!("token {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .json(&serde_json::json!({
            "title": title,
            "body": body,
            "labels": labels
        }))
        .send()
        .await?;
    
    Ok(response.json().await?)
}
```

### Pull Requests

#### List Pull Requests
**Endpoint:** `GET /repos/{owner}/{repo}/pulls`

**Parameters:**
- `state`: `open`, `closed`, `all` (default: `open`)
- `head`: Filter by head branch
- `base`: Filter by base branch
- `sort`: `created`, `updated`, `popularity`, `long-running` (default: `created`)

#### Create Pull Request
**Endpoint:** `POST /repos/{owner}/{repo}/pulls`

**Request Body:**
```json
{
  "title": "Amazing new feature",
  "body": "Please pull this in!",
  "head": "octocat:new-feature",
  "base": "main"
}
```

## Best Practices

### 1. Use Conditional Requests

Use ETags and Last-Modified headers to reduce API calls:

```rust
async fn get_repo_with_etag(
    owner: &str,
    repo: &str,
    etag: Option<&str>,
    token: &str
) -> Result<(serde_json::Value, Option<String>), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut request = client
        .get(&format!("https://api.github.com/repos/{}/{}", owner, repo))
        .header("Authorization", format!("token {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28");
    
    if let Some(e) = etag {
        request = request.header("If-None-Match", e);
    }
    
    let response = request.send().await?;
    
    if response.status() == 304 {
        // Not modified
        return Ok((serde_json::Value::Null, etag.map(|s| s.to_string())));
    }
    
    let new_etag = response.headers()
        .get("etag")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    
    Ok((response.json().await?, new_etag))
}
```

### 2. Implement Exponential Backoff

Handle rate limits and temporary failures:

```rust
use std::time::Duration;
use tokio::time::sleep;

async fn request_with_retry(
    client: &reqwest::Client,
    request: reqwest::RequestBuilder,
    max_retries: u32
) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
    let mut retry_count = 0;
    let mut delay = Duration::from_secs(1);
    
    loop {
        let response = request.try_clone().unwrap().send().await?;
        
        if response.status() == 403 {
            // Rate limit exceeded
            if let Some(reset) = response.headers().get("X-RateLimit-Reset") {
                let reset_time = reset.to_str()?.parse::<u64>()?;
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs();
                let wait_time = reset_time.saturating_sub(now) + 1;
                sleep(Duration::from_secs(wait_time)).await;
                continue;
            }
        }
        
        if response.status().is_server_error() && retry_count < max_retries {
            retry_count += 1;
            sleep(delay).await;
            delay *= 2; // Exponential backoff
            continue;
        }
        
        return Ok(response);
    }
}
```

### 3. Use Webhooks Instead of Polling

Webhooks provide real-time notifications and reduce API calls:

- Set up webhook endpoints in your application
- Register webhooks with GitHub
- Receive events as they happen

### 4. Cache Responses

Cache API responses when appropriate to reduce calls:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

struct ApiCache {
    cache: Arc<RwLock<HashMap<String, (serde_json::Value, std::time::Instant)>>>,
    ttl: Duration,
}

impl ApiCache {
    async fn get(&self, key: &str) -> Option<serde_json::Value> {
        let cache = self.cache.read().await;
        if let Some((value, timestamp)) = cache.get(key) {
            if timestamp.elapsed() < self.ttl {
                return Some(value.clone());
            }
        }
        None
    }
    
    async fn set(&self, key: String, value: serde_json::Value) {
        let mut cache = self.cache.write().await;
        cache.insert(key, (value, std::time::Instant::now()));
    }
}
```

## Error Handling

### Common HTTP Status Codes

- **200 OK:** Request succeeded
- **201 Created:** Resource created successfully
- **204 No Content:** Request succeeded, no content to return
- **304 Not Modified:** Resource not modified (conditional request)
- **400 Bad Request:** Invalid request
- **401 Unauthorized:** Authentication required
- **403 Forbidden:** Access denied (often rate limit)
- **404 Not Found:** Resource not found
- **422 Unprocessable Entity:** Validation failed
- **500 Internal Server Error:** GitHub server error
- **503 Service Unavailable:** Service temporarily unavailable

### Error Response Format

```json
{
  "message": "Validation Failed",
  "errors": [
    {
      "resource": "Issue",
      "field": "title",
      "code": "missing_field"
    }
  ],
  "documentation_url": "https://docs.github.com/rest/reference/issues#create-an-issue"
}
```

### Error Handling Example

```rust
#[derive(Debug)]
enum GitHubError {
    RateLimitExceeded { reset_time: u64 },
    ValidationError { message: String, errors: Vec<String> },
    NotFound,
    Unauthorized,
    ServerError { status: u16 },
    NetworkError(Box<dyn std::error::Error>),
}

async fn handle_api_response(
    response: reqwest::Response
) -> Result<serde_json::Value, GitHubError> {
    let status = response.status();
    
    match status.as_u16() {
        200 | 201 => Ok(response.json().await.map_err(|e| GitHubError::NetworkError(Box::new(e)))?),
        304 => Err(GitHubError::NotFound), // Not modified
        401 => Err(GitHubError::Unauthorized),
        403 => {
            if let Some(reset) = response.headers().get("X-RateLimit-Reset") {
                let reset_time = reset.to_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
                Err(GitHubError::RateLimitExceeded { reset_time })
            } else {
                Err(GitHubError::Unauthorized)
            }
        },
        404 => Err(GitHubError::NotFound),
        422 => {
            let error_data: serde_json::Value = response.json().await
                .map_err(|e| GitHubError::NetworkError(Box::new(e)))?;
            let message = error_data["message"].as_str().unwrap_or("Validation failed").to_string();
            let errors = error_data["errors"]
                .as_array()
                .map(|arr| arr.iter()
                    .map(|e| e["message"].as_str().unwrap_or("").to_string())
                    .collect())
                .unwrap_or_default();
            Err(GitHubError::ValidationError { message, errors })
        },
        _ => Err(GitHubError::ServerError { status: status.as_u16() })
    }
}
```

## Timezones

All timestamps in the GitHub API are returned in UTC time, ISO 8601 format: `YYYY-MM-DDTHH:MM:SSZ`.

**Example:**
```
2022-11-28T12:34:56Z
```

**Parsing Timestamps (Rust):**
```rust
use chrono::{DateTime, Utc};

fn parse_github_timestamp(timestamp: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    DateTime::parse_from_rfc3339(timestamp)
        .map(|dt| dt.with_timezone(&Utc))
}
```

## CORS and JSONP

GitHub API supports CORS (Cross-Origin Resource Sharing) for browser-based applications.

### CORS Headers

GitHub includes the following CORS headers in responses:

```http
Access-Control-Allow-Origin: *
Access-Control-Expose-Headers: ETag, Link, Location, Retry-After, X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Used, X-RateLimit-Resource, X-RateLimit-Reset, X-OAuth-Scopes, X-Accepted-OAuth-Scopes, X-Poll-Interval, X-GitHub-Media-Type, X-GitHub-SSO, X-GitHub-Request-Id, Deprecation, Sunset
```

### JSONP Support

GitHub supports JSONP for GET requests by adding a `callback` query parameter:

```bash
curl "https://api.github.com/user?callback=myCallback"
```

**Response:**
```javascript
myCallback({
  "login": "octocat",
  "id": 1,
  ...
})
```

## Event Types

### Issue Event Types

When working with issue timelines, various event types are possible:

- `added_to_project`
- `assigned`
- `closed`
- `commented`
- `committed`
- `cross-referenced`
- `demilestoned`
- `labeled`
- `locked`
- `milestoned`
- `moved_columns_in_project`
- `referenced`
- `removed_from_project`
- `renamed`
- `reopened`
- `subscribed`
- `transferred`
- `unassigned`
- `unlabeled`
- `unlocked`
- `unsubscribed`

### GitHub Event Types

For webhooks and activity feeds:

- `CreateEvent`
- `DeleteEvent`
- `ForkEvent`
- `IssueCommentEvent`
- `IssuesEvent`
- `PullRequestEvent`
- `PushEvent`
- `ReleaseEvent`
- `WatchEvent`

## Troubleshooting

### Common Issues

1. **401 Unauthorized:**
   - Check token validity
   - Ensure token has required scopes
   - Verify token format in Authorization header

2. **403 Forbidden:**
   - Check rate limits
   - Verify permissions
   - Check if resource exists and is accessible

3. **404 Not Found:**
   - Verify resource path
   - Check if resource exists
   - Verify authentication

4. **422 Unprocessable Entity:**
   - Check request body format
   - Verify required fields
   - Check field validation rules

5. **Rate Limit Exceeded:**
   - Implement exponential backoff
   - Use conditional requests
   - Consider using webhooks
   - Authenticate requests for higher limits

### Debugging Tips

1. **Enable Request Logging:**
```rust
use tracing;

tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

2. **Log Request/Response:**
```rust
let response = client
    .get(url)
    .header("Authorization", format!("token {}", token))
    .send()
    .await?;

tracing::debug!("Status: {}", response.status());
tracing::debug!("Headers: {:?}", response.headers());
let body = response.text().await?;
tracing::debug!("Body: {}", body);
```

3. **Check Rate Limit Status:**
```rust
async fn check_rate_limit_status(token: &str) {
    let response = reqwest::Client::new()
        .get("https://api.github.com/rate_limit")
        .header("Authorization", format!("token {}", token))
        .send()
        .await
        .unwrap();
    
    let status: serde_json::Value = response.json().await.unwrap();
    println!("Rate Limit: {:#}", status);
}
```

## References

### Official Documentation

- [GitHub REST API Overview](https://docs.github.com/en/rest/about-the-rest-api/about-the-rest-api?apiVersion=2022-11-28)
- [Getting Started with the REST API](https://docs.github.com/en/rest/using-the-rest-api/getting-started-with-the-rest-api?apiVersion=2022-11-28)
- [Rate Limits](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api?apiVersion=2022-11-28)
- [Pagination](https://docs.github.com/en/rest/using-the-rest-api/using-pagination-in-the-rest-api?apiVersion=2022-11-28)
- [Best Practices](https://docs.github.com/en/rest/using-the-rest-api/best-practices-for-using-the-rest-api?apiVersion=2022-11-28)
- [Troubleshooting](https://docs.github.com/en/rest/using-the-rest-api/troubleshooting-the-rest-api?apiVersion=2022-11-28)
- [Timezones](https://docs.github.com/en/rest/using-the-rest-api/timezones-and-the-rest-api?apiVersion=2022-11-28)
- [CORS and JSONP](https://docs.github.com/en/rest/using-the-rest-api/using-cors-and-jsonp-to-make-cross-origin-requests)
- [Issue Event Types](https://docs.github.com/en/rest/using-the-rest-api/issue-event-types)
- [GitHub Event Types](https://docs.github.com/en/rest/using-the-rest-api/github-event-types?apiVersion=2022-11-28)

### Authentication

- [Authenticating to the REST API](https://docs.github.com/en/rest/authentication/authenticating-to-the-rest-api?apiVersion=2022-11-28)
- [Keeping API Credentials Secure](https://docs.github.com/en/rest/authentication/keeping-your-api-credentials-secure?apiVersion=2022-11-28)
- [Fine-Grained Personal Access Tokens](https://docs.github.com/en/rest/authentication/endpoints-available-for-fine-grained-personal-access-tokens)
- [Permissions for Fine-Grained PATs](https://docs.github.com/en/rest/authentication/permissions-required-for-fine-grained-personal-access-tokens)

### API Reference

- [Repositories API](https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28)
- [Issues API](https://docs.github.com/en/rest/issues/issues?apiVersion=2022-11-28)
- [Pull Requests API](https://docs.github.com/en/rest/pulls/pulls?apiVersion=2022-11-28)
- [Search API](https://docs.github.com/en/rest/search/search?apiVersion=2022-11-28)

### Additional Resources

- [GitHub API Changelog](https://github.blog/changelog/)
- [GitHub Developer Guide](https://docs.github.com/en/developers)
- [GitHub REST API OpenAPI Specification](https://github.com/github/rest-api-description)

