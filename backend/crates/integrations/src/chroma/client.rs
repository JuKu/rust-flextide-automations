//! Chroma API Client
//! 
//! A client for making requests to the Chroma vector database REST API.

use crate::chroma::error::ChromaError;
use crate::chroma::types::*;
use reqwest::Client;
use tracing::{debug, error, info};

const DEFAULT_CHROMA_BASE_URL: &str = "http://localhost:8000";
const CHROMA_API_VERSION: &str = "v1";

/// Client for interacting with the Chroma vector database REST API
pub struct ChromaClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl ChromaClient {
    /// Create a new Chroma client with default settings (localhost:8000)
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: DEFAULT_CHROMA_BASE_URL.to_string(),
            api_key: None,
        }
    }

    /// Create a new Chroma client with a custom base URL
    pub fn with_base_url(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            api_key: None,
        }
    }

    /// Create a new Chroma client with API key authentication
    pub fn with_api_key(base_url: String, api_key: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            api_key: Some(api_key),
        }
    }

    /// Build the API URL for a given endpoint
    fn api_url(&self, endpoint: &str) -> String {
        format!("{}/api/{}/{}", self.base_url, CHROMA_API_VERSION, endpoint)
    }

    /// Build request headers including authentication if available
    fn build_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );

        if let Some(api_key) = &self.api_key {
            // Chroma uses X-Chroma-Token header for authentication
            let header_name = reqwest::header::HeaderName::from_static("x-chroma-token");
            if let Ok(header_value) = api_key.parse() {
                headers.insert(header_name, header_value);
            }
        }

        headers
    }

    /// Handle HTTP response errors
    fn handle_error(&self, status: reqwest::StatusCode, error_text: String) -> ChromaError {
        error!("Chroma API error: status={}, body={}", status, error_text);

        match status.as_u16() {
            401 => ChromaError::InvalidApiKey,
            404 => {
                // Try to extract collection name from error if possible
                ChromaError::CollectionNotFound(error_text)
            }
            409 => ChromaError::CollectionExists(error_text),
            429 => ChromaError::RateLimitExceeded,
            _ => ChromaError::ApiError(format!("HTTP {}: {}", status, error_text)),
        }
    }

    /// Create a new collection
    pub async fn create_collection(
        &self,
        request: CreateCollectionRequest,
    ) -> Result<Collection, ChromaError> {
        let url = self.api_url("collections");

        debug!("Creating Chroma collection: {}", request.name);

        let response = self
            .client
            .post(&url)
            .headers(self.build_headers())
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(self.handle_error(status, error_text));
        }

        let collection: Collection = response.json().await?;

        info!("Collection created successfully: {}", collection.name);

        Ok(collection)
    }

    /// Get a collection by name
    pub async fn get_collection(&self, name: &str) -> Result<Collection, ChromaError> {
        let url = self.api_url(&format!("collections/{}", name));

        debug!("Getting Chroma collection: {}", name);

        let response = self
            .client
            .get(&url)
            .headers(self.build_headers())
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(self.handle_error(status, error_text));
        }

        let collection: Collection = response.json().await?;

        Ok(collection)
    }

    /// List all collections
    pub async fn list_collections(&self) -> Result<Vec<Collection>, ChromaError> {
        let url = self.api_url("collections");

        debug!("Listing Chroma collections");

        let response = self
            .client
            .get(&url)
            .headers(self.build_headers())
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(self.handle_error(status, error_text));
        }

        let collections: Vec<Collection> = response.json().await?;

        info!("Found {} collections", collections.len());

        Ok(collections)
    }

    /// Delete a collection
    pub async fn delete_collection(&self, name: &str) -> Result<(), ChromaError> {
        let url = self.api_url(&format!("collections/{}", name));

        debug!("Deleting Chroma collection: {}", name);

        let response = self
            .client
            .delete(&url)
            .headers(self.build_headers())
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(self.handle_error(status, error_text));
        }

        info!("Collection deleted successfully: {}", name);

        Ok(())
    }

    /// Add documents to a collection
    pub async fn add_documents(
        &self,
        collection_name: &str,
        request: AddDocumentsRequest,
    ) -> Result<(), ChromaError> {
        let url = self.api_url(&format!("collections/{}/add", collection_name));

        debug!(
            "Adding {} documents to collection: {}",
            request.ids.len(),
            collection_name
        );

        let response = self
            .client
            .post(&url)
            .headers(self.build_headers())
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(self.handle_error(status, error_text));
        }

        info!("Documents added successfully to collection: {}", collection_name);

        Ok(())
    }

    /// Update documents in a collection
    pub async fn update_documents(
        &self,
        collection_name: &str,
        request: UpdateDocumentsRequest,
    ) -> Result<(), ChromaError> {
        let url = self.api_url(&format!("collections/{}/update", collection_name));

        debug!(
            "Updating {} documents in collection: {}",
            request.ids.len(),
            collection_name
        );

        let response = self
            .client
            .post(&url)
            .headers(self.build_headers())
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(self.handle_error(status, error_text));
        }

        info!("Documents updated successfully in collection: {}", collection_name);

        Ok(())
    }

    /// Delete documents from a collection
    pub async fn delete_documents(
        &self,
        collection_name: &str,
        request: DeleteDocumentsRequest,
    ) -> Result<(), ChromaError> {
        let url = self.api_url(&format!("collections/{}/delete", collection_name));

        debug!("Deleting documents from collection: {}", collection_name);

        let response = self
            .client
            .post(&url)
            .headers(self.build_headers())
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(self.handle_error(status, error_text));
        }

        info!("Documents deleted successfully from collection: {}", collection_name);

        Ok(())
    }

    /// Query a collection for similar documents
    pub async fn query(
        &self,
        collection_name: &str,
        request: QueryRequest,
    ) -> Result<QueryResult, ChromaError> {
        let url = self.api_url(&format!("collections/{}/query", collection_name));

        debug!("Querying collection: {}", collection_name);

        let response = self
            .client
            .post(&url)
            .headers(self.build_headers())
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(self.handle_error(status, error_text));
        }

        let result: QueryResult = response.json().await?;

        info!(
            "Query successful: found {} result sets",
            result.ids.len()
        );

        Ok(result)
    }

    /// Get documents from a collection
    pub async fn get_documents(
        &self,
        collection_name: &str,
        request: GetDocumentsRequest,
    ) -> Result<GetDocumentsResult, ChromaError> {
        let url = self.api_url(&format!("collections/{}/get", collection_name));

        debug!("Getting documents from collection: {}", collection_name);

        let response = self
            .client
            .post(&url)
            .headers(self.build_headers())
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(self.handle_error(status, error_text));
        }

        let result: GetDocumentsResult = response.json().await?;

        info!("Retrieved {} documents", result.ids.len());

        Ok(result)
    }

    /// Count documents in a collection
    pub async fn count(&self, collection_name: &str) -> Result<usize, ChromaError> {
        let url = self.api_url(&format!("collections/{}/count", collection_name));

        debug!("Counting documents in collection: {}", collection_name);

        let response = self
            .client
            .post(&url)
            .headers(self.build_headers())
            .json(&serde_json::json!({}))
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(self.handle_error(status, error_text));
        }

        let result: CountResult = response.json().await?;

        info!("Collection {} has {} documents", collection_name, result.count);

        Ok(result.count)
    }

    /// Peek at documents in a collection (get sample)
    pub async fn peek(
        &self,
        collection_name: &str,
        limit: Option<usize>,
    ) -> Result<PeekResult, ChromaError> {
        let mut url = self.api_url(&format!("collections/{}/peek", collection_name));

        if let Some(limit) = limit {
            url = format!("{}?limit={}", url, limit);
        }

        debug!("Peeking at collection: {}", collection_name);

        let response = self
            .client
            .get(&url)
            .headers(self.build_headers())
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(self.handle_error(status, error_text));
        }

        let result: PeekResult = response.json().await?;

        Ok(result)
    }
}

impl Default for ChromaClient {
    fn default() -> Self {
        Self::new()
    }
}

