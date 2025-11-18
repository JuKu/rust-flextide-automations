//! GitHub API Client
//! 
//! A client for making requests to the GitHub API.

use crate::github::error::GitHubError;
use crate::github::types::*;
use reqwest::Client;
use reqwest::StatusCode;
use tracing::{debug, error, info, warn};

const GITHUB_API_BASE: &str = "https://api.github.com";
const GITHUB_API_VERSION: &str = "2022-11-28";

/// Client for interacting with the GitHub API
pub struct GitHubClient {
    client: Client,
    token: Option<String>,
    base_url: String,
}

impl GitHubClient {
    /// Create a new GitHub client without authentication
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            token: None,
            base_url: GITHUB_API_BASE.to_string(),
        }
    }

    /// Create a new GitHub client with authentication token
    pub fn with_token(token: String) -> Self {
        Self {
            client: Client::new(),
            token: Some(token),
            base_url: GITHUB_API_BASE.to_string(),
        }
    }

    /// Create a new GitHub client with a custom base URL (useful for GitHub Enterprise)
    pub fn with_base_url(token: Option<String>, base_url: String) -> Self {
        Self {
            client: Client::new(),
            token,
            base_url,
        }
    }

    /// Build request headers with authentication if token is available
    fn build_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::ACCEPT,
            "application/vnd.github+json".parse().unwrap(),
        );
        headers.insert(
            reqwest::header::USER_AGENT,
            "Flextide-Integration/1.0".parse().unwrap(),
        );
        headers.insert(
            reqwest::header::HeaderName::from_static("x-github-api-version"),
            reqwest::header::HeaderValue::from_static(GITHUB_API_VERSION),
        );

        if let Some(ref token) = self.token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", token).parse().unwrap(),
            );
        }

        headers
    }

    /// Handle HTTP response errors and rate limiting
    async fn handle_response<T>(response: reqwest::Response) -> Result<T, GitHubError>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();
        let headers = response.headers().clone();

        match status {
            StatusCode::OK | StatusCode::CREATED | StatusCode::NO_CONTENT => {
                let text = response.text().await?;
                serde_json::from_str(&text).map_err(|e| {
                    error!("Failed to parse JSON response: {}", e);
                    GitHubError::JsonError(e)
                })
            }
            StatusCode::UNAUTHORIZED => {
                let text = response.text().await.unwrap_or_default();
                Err(GitHubError::AuthenticationError(format!(
                    "Unauthorized: {}",
                    text
                )))
            }
            StatusCode::FORBIDDEN => {
                // Check if it's a rate limit error
                if let Some(retry_after) = headers.get("retry-after") {
                    let retry_after_str = retry_after.to_str().unwrap_or("unknown");
                    let text = response.text().await.unwrap_or_default();
                    warn!("Rate limit exceeded. Retry after: {} seconds", retry_after_str);
                    Err(GitHubError::RateLimitError(format!(
                        "Rate limit exceeded. Retry after: {} seconds. Response: {}",
                        retry_after_str, text
                    )))
                } else {
                    let text = response.text().await.unwrap_or_default();
                    Err(GitHubError::ApiError(format!("Forbidden: {}", text)))
                }
            }
            StatusCode::NOT_FOUND => {
                let text = response.text().await.unwrap_or_default();
                Err(GitHubError::NotFound(format!("Not found: {}", text)))
            }
            StatusCode::TOO_MANY_REQUESTS => {
                let retry_after = headers
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("unknown");
                let text = response.text().await.unwrap_or_default();
                warn!("Rate limit exceeded. Retry after: {} seconds", retry_after);
                Err(GitHubError::RateLimitError(format!(
                    "Rate limit exceeded. Retry after: {} seconds. Response: {}",
                    retry_after, text
                )))
            }
            _ => {
                let text = response.text().await.unwrap_or_default();
                error!("Unexpected status {}: {}", status, text);
                Err(GitHubError::ApiError(format!(
                    "HTTP {}: {}",
                    status, text
                )))
            }
        }
    }

    /// Get all organizations (paginated)
    /// 
    /// Returns a list of all organizations. This endpoint supports pagination.
    /// Fine-grained personal access tokens and classic tokens are both supported.
    /// 
    /// # Arguments
    /// 
    /// * `per_page` - Number of results per page (1-100, default: 30)
    /// * `since` - Only show organizations updated after this time (optional)
    /// 
    /// # Returns
    /// 
    /// A vector of `OrganizationSimple` objects
    pub async fn list_organizations(
        &self,
        per_page: Option<u32>,
        since: Option<u64>,
    ) -> Result<Vec<OrganizationSimple>, GitHubError> {
        let mut url = format!("{}/organizations", self.base_url);
        let mut query_params = vec![];

        if let Some(per_page) = per_page {
            query_params.push(format!("per_page={}", per_page.min(100).max(1)));
        }

        if let Some(since) = since {
            query_params.push(format!("since={}", since));
        }

        if !query_params.is_empty() {
            url.push_str("?");
            url.push_str(&query_params.join("&"));
        }

        let mut all_organizations = Vec::new();
        let mut current_url = Some(url);

        while let Some(url) = current_url {
            debug!("Fetching organizations from: {}", url);

            let response = self
                .client
                .get(&url)
                .headers(self.build_headers())
                .send()
                .await?;

            // Extract headers before consuming response
            let headers = response.headers().clone();
            let organizations: Vec<OrganizationSimple> = Self::handle_response(response).await?;
            all_organizations.extend(organizations);

            // Check for pagination link in headers
            current_url = self.get_next_page_url(&headers);
        }

        info!("Fetched {} organizations", all_organizations.len());
        Ok(all_organizations)
    }

    /// Get a single organization by name
    /// 
    /// Returns detailed information about a specific organization.
    /// Fine-grained personal access tokens and classic tokens are both supported.
    /// 
    /// # Arguments
    /// 
    /// * `org` - The organization name (e.g., "github")
    /// 
    /// # Returns
    /// 
    /// An `OrganizationFull` object with complete organization details
    pub async fn get_organization(&self, org: &str) -> Result<OrganizationFull, GitHubError> {
        let url = format!("{}/orgs/{}", self.base_url, org);
        debug!("Fetching organization: {}", org);

        let response = self
            .client
            .get(&url)
            .headers(self.build_headers())
            .send()
            .await?;

        let organization: OrganizationFull = Self::handle_response(response).await?;
        info!("Fetched organization: {}", organization.login);
        Ok(organization)
    }

    /// List repositories for the authenticated user (paginated)
    /// 
    /// Returns a list of repositories that belong to the authenticated user.
    /// This includes personal repositories (not belonging to an organization).
    /// Fine-grained personal access tokens and classic tokens are both supported.
    /// 
    /// # Arguments
    /// 
    /// * `repo_type` - Filter by repository type: all, owner, member (default: all)
    ///   - `all`: All repositories the user has access to
    ///   - `owner`: Only repositories owned by the user
    ///   - `member`: Only repositories where the user is a member
    /// * `sort` - Sort by: created, updated, pushed, full_name (default: full_name)
    /// * `direction` - Sort direction: asc, desc (default: desc when not using full_name, otherwise asc)
    /// * `per_page` - Number of results per page (1-100, default: 30)
    /// 
    /// # Returns
    /// 
    /// A vector of `Repository` objects
    /// 
    /// # Note
    /// 
    /// Requires authentication. The token must have appropriate permissions.
    pub async fn list_user_repositories(
        &self,
        repo_type: Option<&str>,
        sort: Option<&str>,
        direction: Option<&str>,
        per_page: Option<u32>,
    ) -> Result<Vec<Repository>, GitHubError> {
        let mut url = format!("{}/user/repos", self.base_url);
        let mut query_params = vec![];

        if let Some(repo_type) = repo_type {
            query_params.push(format!("type={}", repo_type));
        }

        if let Some(sort) = sort {
            query_params.push(format!("sort={}", sort));
        }

        if let Some(direction) = direction {
            query_params.push(format!("direction={}", direction));
        }

        if let Some(per_page) = per_page {
            query_params.push(format!("per_page={}", per_page.min(100).max(1)));
        }

        if !query_params.is_empty() {
            url.push_str("?");
            url.push_str(&query_params.join("&"));
        }

        let mut all_repositories = Vec::new();
        let mut current_url = Some(url);

        while let Some(url) = current_url {
            debug!("Fetching user repositories from: {}", url);

            let response = self
                .client
                .get(&url)
                .headers(self.build_headers())
                .send()
                .await?;

            // Extract headers before consuming response
            let headers = response.headers().clone();
            let repositories: Vec<Repository> = Self::handle_response(response).await?;
            all_repositories.extend(repositories);

            // Check for pagination link in headers
            current_url = self.get_next_page_url(&headers);
        }

        info!("Fetched {} repositories for authenticated user", all_repositories.len());
        Ok(all_repositories)
    }

    /// List all repositories for an organization (paginated)
    /// 
    /// Returns a list of repositories for the specified organization.
    /// Fine-grained personal access tokens and classic tokens are both supported.
    /// 
    /// # Arguments
    /// 
    /// * `org` - The organization name (e.g., "github")
    /// * `repo_type` - Filter by repository type: all, public, private, forks, sources, member (default: all)
    /// * `sort` - Sort by: created, updated, pushed, full_name (default: full_name)
    /// * `direction` - Sort direction: asc, desc (default: asc when using full_name, otherwise desc)
    /// * `per_page` - Number of results per page (1-100, default: 30)
    /// 
    /// # Returns
    /// 
    /// A vector of `Repository` objects
    pub async fn list_organization_repositories(
        &self,
        org: &str,
        repo_type: Option<&str>,
        sort: Option<&str>,
        direction: Option<&str>,
        per_page: Option<u32>,
    ) -> Result<Vec<Repository>, GitHubError> {
        let mut url = format!("{}/orgs/{}/repos", self.base_url, org);
        let mut query_params = vec![];

        if let Some(repo_type) = repo_type {
            query_params.push(format!("type={}", repo_type));
        }

        if let Some(sort) = sort {
            query_params.push(format!("sort={}", sort));
        }

        if let Some(direction) = direction {
            query_params.push(format!("direction={}", direction));
        }

        if let Some(per_page) = per_page {
            query_params.push(format!("per_page={}", per_page.min(100).max(1)));
        }

        if !query_params.is_empty() {
            url.push_str("?");
            url.push_str(&query_params.join("&"));
        }

        let mut all_repositories = Vec::new();
        let mut current_url = Some(url);

        while let Some(url) = current_url {
            debug!("Fetching repositories from: {}", url);

            let response = self
                .client
                .get(&url)
                .headers(self.build_headers())
                .send()
                .await?;

            // Extract headers before consuming response
            let headers = response.headers().clone();
            let repositories: Vec<Repository> = Self::handle_response(response).await?;
            all_repositories.extend(repositories);

            // Check for pagination link in headers
            current_url = self.get_next_page_url(&headers);
        }

        info!("Fetched {} repositories for organization: {}", all_repositories.len(), org);
        Ok(all_repositories)
    }

    /// Get a single repository by owner and name
    /// 
    /// Returns detailed information about a specific repository.
    /// Fine-grained personal access tokens and classic tokens are both supported.
    /// 
    /// # Arguments
    /// 
    /// * `owner` - The repository owner (user or organization name)
    /// * `repo` - The repository name
    /// 
    /// # Returns
    /// 
    /// A `RepositoryFull` object with complete repository details
    pub async fn get_repository(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<RepositoryFull, GitHubError> {
        let url = format!("{}/repos/{}/{}", self.base_url, owner, repo);
        debug!("Fetching repository: {}/{}", owner, repo);

        let response = self
            .client
            .get(&url)
            .headers(self.build_headers())
            .send()
            .await?;

        let repository: RepositoryFull = Self::handle_response(response).await?;
        info!("Fetched repository: {}", repository.full_name);
        Ok(repository)
    }

    /// Create a repository for the authenticated user
    /// 
    /// Creates a new repository in the authenticated user's account.
    /// Fine-grained personal access tokens and classic tokens are both supported.
    /// 
    /// # Arguments
    /// 
    /// * `request` - The repository creation request
    /// 
    /// # Returns
    /// 
    /// A `RepositoryFull` object with the created repository details
    /// 
    /// # Note
    /// 
    /// Requires authentication. The token must have appropriate permissions.
    pub async fn create_user_repository(
        &self,
        request: CreateRepositoryRequest,
    ) -> Result<RepositoryFull, GitHubError> {
        let url = format!("{}/user/repos", self.base_url);
        debug!("Creating user repository: {}", request.name);

        let response = self
            .client
            .post(&url)
            .headers(self.build_headers())
            .json(&request)
            .send()
            .await?;

        let repository: RepositoryFull = Self::handle_response(response).await?;
        info!("Created user repository: {}", repository.full_name);
        Ok(repository)
    }

    /// Create a repository for an organization
    /// 
    /// Creates a new repository in the specified organization.
    /// Fine-grained personal access tokens and classic tokens are both supported.
    /// 
    /// # Arguments
    /// 
    /// * `org` - The organization name
    /// * `request` - The repository creation request
    /// 
    /// # Returns
    /// 
    /// A `RepositoryFull` object with the created repository details
    /// 
    /// # Note
    /// 
    /// Requires authentication. The token must have appropriate permissions for the organization.
    pub async fn create_organization_repository(
        &self,
        org: &str,
        request: CreateRepositoryRequest,
    ) -> Result<RepositoryFull, GitHubError> {
        let url = format!("{}/orgs/{}/repos", self.base_url, org);
        debug!("Creating organization repository: {}/{}", org, request.name);

        let response = self
            .client
            .post(&url)
            .headers(self.build_headers())
            .json(&request)
            .send()
            .await?;

        let repository: RepositoryFull = Self::handle_response(response).await?;
        info!("Created organization repository: {}", repository.full_name);
        Ok(repository)
    }

    /// Extract next page URL from Link header (for pagination)
    fn get_next_page_url(&self, headers: &reqwest::header::HeaderMap) -> Option<String> {
        headers
            .get("link")
            .and_then(|link| link.to_str().ok())
            .and_then(|link_str| {
                // Parse Link header: <https://api.github.com/organizations?page=2>; rel="next"
                for link_part in link_str.split(',') {
                    let parts: Vec<&str> = link_part.split(';').collect();
                    if parts.len() == 2 {
                        let url = parts[0].trim().trim_start_matches('<').trim_end_matches('>');
                        let rel = parts[1].trim();
                        if rel.contains("rel=\"next\"") || rel.contains("rel='next'") {
                            return Some(url.to_string());
                        }
                    }
                }
                None
            })
    }
}

impl Default for GitHubClient {
    fn default() -> Self {
        Self::new()
    }
}

