//! GitHub API Client
//! 
//! A client for making requests to the GitHub API.

use crate::github::error::GitHubError;
use crate::github::types::*;
use reqwest::Client;
use tracing::{debug, error, info};

const GITHUB_API_BASE: &str = "https://api.github.com";

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
            "application/vnd.github.v3+json".parse().unwrap(),
        );
        headers.insert(
            reqwest::header::USER_AGENT,
            "Flextide-Integration/1.0".parse().unwrap(),
        );

        if let Some(ref token) = self.token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", token).parse().unwrap(),
            );
        }

        headers
    }
}

impl Default for GitHubClient {
    fn default() -> Self {
        Self::new()
    }
}

