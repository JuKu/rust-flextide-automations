//! GitLab API Client
//! 
//! A client for making requests to the GitLab API.

use crate::gitlab::error::GitLabError;
use crate::gitlab::types::*;
use reqwest::Client;
use reqwest::StatusCode;
use tracing::{debug, error, info};

const GITLAB_API_BASE: &str = "https://gitlab.com/api/v4";

/// Client for interacting with the GitLab API
pub struct GitLabClient {
    client: Client,
    token: Option<String>,
    base_url: String,
}

impl GitLabClient {
    /// Create a new GitLab client without authentication
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            token: None,
            base_url: GITLAB_API_BASE.to_string(),
        }
    }

    /// Create a new GitLab client with authentication token
    pub fn with_token(token: String) -> Self {
        Self {
            client: Client::new(),
            token: Some(token),
            base_url: GITLAB_API_BASE.to_string(),
        }
    }

    /// Create a new GitLab client with a custom base URL (useful for self-hosted GitLab)
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
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        if let Some(token) = &self.token {
            headers.insert(
                reqwest::header::HeaderName::from_static("private-token"),
                reqwest::header::HeaderValue::from_str(token)
                    .expect("Token should be valid header value"),
            );
        }

        headers
    }

    /// Build the full URL for an API endpoint
    fn build_url(&self, endpoint: &str) -> String {
        format!("{}/{}", self.base_url, endpoint.trim_start_matches('/'))
    }

    /// Handle API response and extract error information if needed
    async fn handle_response<T: for<'de> Deserialize<'de>>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, GitLabError> {
        let status = response.status();

        if status.is_success() {
            let body = response.text().await?;
            debug!("GitLab API response: {}", body);
            serde_json::from_str(&body).map_err(GitLabError::JsonError)
        } else {
            let error_text = response.text().await?;
            error!("GitLab API error ({}): {}", status, error_text);

            match status {
                StatusCode::UNAUTHORIZED => Err(GitLabError::AuthenticationError(format!(
                    "Unauthorized: {}",
                    error_text
                ))),
                StatusCode::FORBIDDEN => Err(GitLabError::AuthenticationError(format!(
                    "Forbidden: {}",
                    error_text
                ))),
                StatusCode::NOT_FOUND => Err(GitLabError::NotFound(format!(
                    "Resource not found: {}",
                    error_text
                ))),
                StatusCode::TOO_MANY_REQUESTS => Err(GitLabError::RateLimitError(format!(
                    "Rate limit exceeded: {}",
                    error_text
                ))),
                StatusCode::UNPROCESSABLE_ENTITY => Err(GitLabError::InvalidRequest(format!(
                    "Invalid request: {}",
                    error_text
                ))),
                _ => Err(GitLabError::ApiError(format!(
                    "API error ({}): {}",
                    status, error_text
                ))),
            }
        }
    }

    /// Extract pagination information from response headers
    #[allow(dead_code)]
    fn extract_pagination_info(&self, response: &reqwest::Response) -> PaginationInfo {
        PaginationInfo {
            total: response
                .headers()
                .get("x-total")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse().ok()),
            total_pages: response
                .headers()
                .get("x-total-pages")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse().ok()),
            per_page: response
                .headers()
                .get("x-per-page")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse().ok()),
            page: response
                .headers()
                .get("x-page")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse().ok()),
            next_page: response
                .headers()
                .get("x-next-page")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse().ok()),
            prev_page: response
                .headers()
                .get("x-prev-page")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse().ok()),
        }
    }

    /// Get the current authenticated user
    pub async fn get_current_user(&self) -> Result<User, GitLabError> {
        let url = self.build_url("user");
        info!("Getting current user from GitLab API");

        let response = self
            .client
            .get(&url)
            .headers(self.build_headers())
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// List all projects
    pub async fn list_projects(
        &self,
        pagination: Option<PaginationParams>,
    ) -> Result<Vec<Project>, GitLabError> {
        let url = self.build_url("projects");
        info!("Listing projects from GitLab API");

        let mut request = self.client.get(&url).headers(self.build_headers());

        if let Some(pagination) = pagination {
            if let Some(page) = pagination.page {
                request = request.query(&[("page", page.to_string())]);
            }
            if let Some(per_page) = pagination.per_page {
                request = request.query(&[("per_page", per_page.to_string())]);
            }
        }

        let response = request.send().await?;
        self.handle_response(response).await
    }

    /// Get a specific project by ID or path
    pub async fn get_project(&self, project_id: &str) -> Result<Project, GitLabError> {
        let url = self.build_url(&format!("projects/{}", project_id));
        info!("Getting project {} from GitLab API", project_id);

        let response = self
            .client
            .get(&url)
            .headers(self.build_headers())
            .send()
            .await?;

        self.handle_response(response).await
    }
}

impl Default for GitLabClient {
    fn default() -> Self {
        Self::new()
    }
}

use serde::Deserialize;

