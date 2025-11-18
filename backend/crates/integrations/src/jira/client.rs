//! Jira API Client

use crate::jira::error::JiraError;
use crate::jira::types::*;
use reqwest::Client;
use tracing::{debug, error};

/// Client for interacting with the Jira API
pub struct JiraClient {
    base_url: String,
    client: Client,
    email: String,
    auth_token: String,
}

impl JiraClient {
    /// Create a new Jira client
    /// 
    /// # Arguments
    /// * `base_url` - The base URL of your Jira instance
    ///   - For Jira Cloud: "https://your-domain.atlassian.net"
    ///   - For Jira Data Center: "https://your-jira-server.com" or "https://your-jira-server.com/jira"
    /// * `email` - Your Jira account email address
    /// * `auth_token` - Your Jira API token
    /// 
    /// # Note
    /// This client supports both Jira Cloud and Jira Data Center. Ensure the base URL
    /// includes the correct path (e.g., "/jira" for Data Center if required).
    pub fn new(base_url: String, email: String, auth_token: String) -> Self {
        Self {
            base_url,
            client: Client::new(),
            email,
            auth_token,
        }
    }

    /// Get all visible projects for the user in a paginated way
    /// 
    /// # Arguments
    /// * `start_at` - The index of the first item to return (default: 0)
    /// * `max_results` - The maximum number of items to return (default: 100)
    /// 
    /// # Returns
    /// A `ProjectSearchResponse` containing the paginated list of projects
    pub async fn get_projects(
        &self,
        start_at: Option<usize>,
        max_results: Option<usize>,
    ) -> Result<ProjectSearchResponse, JiraError> {
        let start_at = start_at.unwrap_or(0);
        let max_results = max_results.unwrap_or(100);
        
        let url = format!("{}/rest/api/3/project/search", self.base_url);
        
        debug!("Fetching projects from Jira: start_at={}, max_results={}", start_at, max_results);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("startAt", start_at.to_string()),
                ("maxResults", max_results.to_string()),
            ])
            .basic_auth(&self.email, Some(&self.auth_token))
            .header("Accept", "application/json")
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Jira API error: status={}, body={}", status, error_text);

            return match status.as_u16() {
                400 => Err(JiraError::InvalidRequest(format!(
                    "Bad request: {}",
                    error_text
                ))),
                401 => Err(JiraError::AuthenticationError(format!(
                    "Authentication failed: {}",
                    error_text
                ))),
                404 => Err(JiraError::ApiError(format!(
                    "Resource not found: {}",
                    error_text
                ))),
                _ => Err(JiraError::ApiError(format!(
                    "HTTP {}: {}",
                    status, error_text
                ))),
            };
        }

        let search_response: ProjectSearchResponse = response.json().await?;
        
        debug!(
            "Projects fetched successfully: total={}, returned={}",
            search_response.total,
            search_response.values.len()
        );

        Ok(search_response)
    }

    /// Get detailed information about a single project
    /// 
    /// # Arguments
    /// * `project_id_or_key` - The project ID or key (e.g., "EX" or "10000")
    /// 
    /// # Returns
    /// A `ProjectDetails` containing all expanded project information
    pub async fn get_project(
        &self,
        project_id_or_key: &str,
    ) -> Result<ProjectDetails, JiraError> {
        let url = format!("{}/rest/api/3/project/{}", self.base_url, project_id_or_key);
        
        debug!("Fetching project details from Jira: project={}", project_id_or_key);

        let response = self
            .client
            .get(&url)
            .query(&[("expand", "*")])
            .basic_auth(&self.email, Some(&self.auth_token))
            .header("Accept", "application/json")
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Jira API error: status={}, body={}", status, error_text);

            return match status.as_u16() {
                400 => Err(JiraError::InvalidRequest(format!(
                    "Bad request: {}",
                    error_text
                ))),
                401 => Err(JiraError::AuthenticationError(format!(
                    "Authentication failed: {}",
                    error_text
                ))),
                404 => Err(JiraError::ApiError(format!(
                    "Project not found: {}",
                    project_id_or_key
                ))),
                _ => Err(JiraError::ApiError(format!(
                    "HTTP {}: {}",
                    status, error_text
                ))),
            };
        }

        let project: ProjectDetails = response.json().await?;
        
        debug!(
            "Project details fetched successfully: key={}, name={}",
            project.key, project.name
        );

        Ok(project)
    }
}

