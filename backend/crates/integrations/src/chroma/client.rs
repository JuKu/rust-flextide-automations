//! Chroma API Client
//! 
//! A client for making requests to the Chroma vector database REST API.

use crate::chroma::error::ChromaError;
use crate::chroma::types::*;
use reqwest::Client;
use tracing::{debug, error, info};

const DEFAULT_CHROMA_BASE_URL: &str = "http://localhost:8000";
const CHROMA_API_VERSION: &str = "v2";

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

    /// Build the API URL for a given endpoint (API v2 with tenant/database)
    fn api_url(&self, tenant: &str, database: &str, endpoint: &str) -> String {
        format!(
            "{}/api/{}/tenants/{}/databases/{}/{}",
            self.base_url, CHROMA_API_VERSION, tenant, database, endpoint
        )
    }

    /// Build the API URL for a tenant-specific endpoint
    fn tenant_api_url(&self, tenant: &str, _endpoint: &str) -> String {
        format!(
            "{}/api/{}/tenants/{}",
            self.base_url, CHROMA_API_VERSION, tenant
        )
    }

    /// Build the API URL for a general v2 endpoint (without tenant/database)
    fn v2_api_url(&self, endpoint: &str) -> String {
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

    /// Build request headers from ChromaCredentials
    pub fn build_headers_from_credentials(creds: &crate::chroma::types::ChromaCredentials) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );

        // Add authentication header based on credentials
        if creds.auth_method == "token" && !creds.auth_token.is_empty() {
            let header_name = if creds.token_transport_header.to_lowercase() == "authorization" {
                reqwest::header::AUTHORIZATION
            } else {
                // Try to parse custom header name
                reqwest::header::HeaderName::try_from(creds.token_transport_header.as_str())
                    .unwrap_or_else(|_| reqwest::header::HeaderName::from_static("x-chroma-token"))
            };

            let header_value = if creds.token_transport_header.to_lowercase() == "authorization" {
                format!("{}{}", creds.token_prefix, creds.auth_token)
            } else {
                creds.auth_token.clone()
            };

            if let Ok(value) = header_value.parse() {
                headers.insert(header_name, value);
            }
        } else if creds.auth_method == "basic_auth" && !creds.auth_token.is_empty() {
            // For basic_auth, auth_token should be in format "username:password"
            // We'll encode it as Basic Auth header
            use base64::Engine;
            let encoded = base64::engine::general_purpose::STANDARD.encode(&creds.auth_token);
            let header_value = format!("Basic {}", encoded);
            if let Ok(value) = header_value.parse() {
                headers.insert(reqwest::header::AUTHORIZATION, value);
            }
        }

        // Add additional headers
        for (key, value) in &creds.additional_headers {
            if let (Ok(header_name), Ok(header_value)) = (
                reqwest::header::HeaderName::try_from(key.as_str()),
                value.parse(),
            ) {
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

    /// Create a new collection (API v2 - requires tenant and database)
    pub async fn create_collection_v2(
        &self,
        tenant: &str,
        database: &str,
        request: CreateCollectionRequest,
    ) -> Result<Collection, ChromaError> {
        let url = self.api_url(tenant, database, "collections");

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

    /// Get a collection by name (API v2 - requires tenant and database)
    pub async fn get_collection(&self, tenant: &str, database: &str, name: &str) -> Result<Collection, ChromaError> {
        let url = self.api_url(tenant, database, &format!("collections/{}", name));

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

    /// List all collections (API v2 - requires tenant and database)
    pub async fn list_collections(&self, tenant: &str, database: &str) -> Result<Vec<Collection>, ChromaError> {
        let url = self.api_url(tenant, database, "collections");

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

    /// Delete a collection (API v2 - requires tenant and database)
    pub async fn delete_collection(&self, tenant: &str, database: &str, name: &str) -> Result<(), ChromaError> {
        let url = self.api_url(tenant, database, &format!("collections/{}", name));

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

    /// Add documents to a collection (API v2 - requires tenant and database)
    pub async fn add_documents(
        &self,
        tenant: &str,
        database: &str,
        collection_name: &str,
        request: AddDocumentsRequest,
    ) -> Result<(), ChromaError> {
        let url = self.api_url(tenant, database, &format!("collections/{}/add", collection_name));

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

    /// Update documents in a collection (API v2 - requires tenant and database)
    pub async fn update_documents(
        &self,
        tenant: &str,
        database: &str,
        collection_name: &str,
        request: UpdateDocumentsRequest,
    ) -> Result<(), ChromaError> {
        let url = self.api_url(tenant, database, &format!("collections/{}/update", collection_name));

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

    /// Delete documents from a collection (API v2 - requires tenant and database)
    pub async fn delete_documents(
        &self,
        tenant: &str,
        database: &str,
        collection_name: &str,
        request: DeleteDocumentsRequest,
    ) -> Result<(), ChromaError> {
        let url = self.api_url(tenant, database, &format!("collections/{}/delete", collection_name));

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

    /// Query a collection for similar documents (API v2 - requires tenant and database)
    pub async fn query(
        &self,
        tenant: &str,
        database: &str,
        collection_name: &str,
        request: QueryRequest,
    ) -> Result<QueryResult, ChromaError> {
        let url = self.api_url(tenant, database, &format!("collections/{}/query", collection_name));

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

    /// Get documents from a collection (API v2 - requires tenant and database)
    pub async fn get_documents(
        &self,
        tenant: &str,
        database: &str,
        collection_name: &str,
        request: GetDocumentsRequest,
    ) -> Result<GetDocumentsResult, ChromaError> {
        let url = self.api_url(tenant, database, &format!("collections/{}/get", collection_name));

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

    /// Count documents in a collection (API v2 - requires tenant and database)
    pub async fn count(&self, tenant: &str, database: &str, collection_name: &str) -> Result<usize, ChromaError> {
        let url = self.api_url(tenant, database, &format!("collections/{}/count", collection_name));

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

    /// Peek at documents in a collection (get sample) (API v2 - requires tenant and database)
    pub async fn peek(
        &self,
        tenant: &str,
        database: &str,
        collection_name: &str,
        limit: Option<usize>,
    ) -> Result<PeekResult, ChromaError> {
        let mut url = self.api_url(tenant, database, &format!("collections/{}/peek", collection_name));

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

    /// Check if a tenant exists
    /// 
    /// GET /api/v2/tenants/{tenant_name}
    pub async fn tenant_exists(&self, tenant_name: &str) -> Result<bool, ChromaError> {
        let url = self.tenant_api_url(tenant_name, "");

        debug!("Checking if tenant exists: {}", tenant_name);

        let response = self
            .client
            .get(&url)
            .headers(self.build_headers())
            .send()
            .await?;

        let status = response.status();

        match status.as_u16() {
            200 => Ok(true),
            404 => Ok(false),
            _ => {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                Err(self.handle_error(status, error_text))
            }
        }
    }

    /// Get user identity (returns available tenants and databases)
    /// 
    /// GET /api/v2/auth/identity
    pub async fn get_user_identity(&self) -> Result<serde_json::Value, ChromaError> {
        let url = self.v2_api_url("auth/identity");

        debug!("Getting user identity");

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

        let identity: serde_json::Value = response.json().await?;
        Ok(identity)
    }

    /// List collections for a tenant/database (API v2)
    /// 
    /// GET /api/v2/tenants/{tenant}/databases/{database}/collections
    pub async fn list_collections_v2(
        &self,
        tenant: &str,
        database: &str,
    ) -> Result<Vec<Collection>, ChromaError> {
        let url = self.api_url(tenant, database, "collections");

        debug!("Listing Chroma collections for tenant={}, database={}", tenant, database);

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

    /// List collections for a tenant/database using credentials (API v2)
    /// 
    /// GET /api/v2/tenants/{tenant}/databases/{database}/collections
    pub async fn list_collections_v2_with_credentials(
        creds: &crate::chroma::types::ChromaCredentials,
        tenant: &str,
        database: &str,
    ) -> Result<Vec<Collection>, ChromaError> {
        let client = Client::new();
        let url = format!(
            "{}/api/{}/tenants/{}/databases/{}/collections",
            creds.base_url, creds.api_version, tenant, database
        );

        debug!("Listing Chroma collections for tenant={}, database={}", tenant, database);

        let headers = Self::build_headers_from_credentials(creds);

        let response = client
            .get(&url)
            .headers(headers)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Self::handle_error_static(status, error_text));
        }

        let collections: Vec<Collection> = response.json().await?;

        info!("Found {} collections", collections.len());

        Ok(collections)
    }

    /// Check if a tenant exists using credentials
    /// 
    /// GET /api/v2/tenants/{tenant_name}
    pub async fn tenant_exists_with_credentials(
        creds: &crate::chroma::types::ChromaCredentials,
        tenant_name: &str,
    ) -> Result<bool, ChromaError> {
        let client = Client::new();
        let url = format!(
            "{}/api/{}/tenants/{}",
            creds.base_url, creds.api_version, tenant_name
        );

        debug!("Checking if tenant exists: {}", tenant_name);

        let headers = Self::build_headers_from_credentials(creds);

        let response = client
            .get(&url)
            .headers(headers)
            .send()
            .await?;

        let status = response.status();

        match status.as_u16() {
            200 => Ok(true),
            404 => Ok(false),
            _ => {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                Err(Self::handle_error_static(status, error_text))
            }
        }
    }

    /// Test connection to Chroma server and verify tenant/database access
    /// 
    /// This method:
    /// 1. Checks if the tenant exists by calling GET /api/{version}/tenants/{tenant}
    /// 2. If database is provided, checks if the database exists by calling GET /api/{version}/tenants/{tenant}/databases/{database}
    ///    Otherwise, lists databases by calling GET /api/{version}/tenants/{tenant}/databases
    pub async fn test_connection_with_credentials(
        creds: &crate::chroma::types::ChromaCredentials,
    ) -> Result<(), ChromaError> {
        let client = Client::new();
        
        // Step 1: Check if tenant exists
        let tenant_url = format!(
            "{}/api/{}/tenants/{}",
            creds.base_url, creds.api_version, creds.tenant_name
        );
        
        debug!("Testing Chroma connection: checking tenant {}", creds.tenant_name);
        
        let headers = Self::build_headers_from_credentials(creds);
        
        let response = client
            .get(&tenant_url)
            .headers(headers.clone())
            .send()
            .await?;
        
        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Self::handle_error_static(status, error_text));
        }
        
        // Step 2: Check database if provided
        if !creds.database_name.is_empty() {
            let db_url = format!(
                "{}/api/{}/tenants/{}/databases/{}",
                creds.base_url, creds.api_version, creds.tenant_name, creds.database_name
            );
            
            debug!("Testing Chroma connection: checking database {}", creds.database_name);
            
            let db_response = client
                .get(&db_url)
                .headers(headers)
                .send()
                .await?;
            
            let db_status = db_response.status();
            
            if !db_status.is_success() {
                let error_text = db_response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                return Err(Self::handle_error_static(db_status, error_text));
            }
        } else {
            // If no database name, try to list databases
            let db_list_url = format!(
                "{}/api/{}/tenants/{}/databases",
                creds.base_url, creds.api_version, creds.tenant_name
            );
            
            debug!("Testing Chroma connection: listing databases");
            
            let db_response = client
                .get(&db_list_url)
                .headers(headers)
                .send()
                .await?;
            
            let db_status = db_response.status();
            
            if !db_status.is_success() {
                let error_text = db_response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                return Err(Self::handle_error_static(db_status, error_text));
            }
        }
        
        info!("Chroma connection test successful");
        Ok(())
    }

    /// Static version of handle_error for use in static methods
    fn handle_error_static(status: reqwest::StatusCode, error_text: String) -> ChromaError {
        error!("Chroma API error: status={}, body={}", status, error_text);

        match status.as_u16() {
            401 => ChromaError::InvalidApiKey,
            404 => {
                ChromaError::CollectionNotFound(error_text)
            }
            409 => ChromaError::CollectionExists(error_text),
            429 => ChromaError::RateLimitExceeded,
            _ => ChromaError::ApiError(format!("HTTP {}: {}", status, error_text)),
        }
    }
}

impl Default for ChromaClient {
    fn default() -> Self {
        Self::new()
    }
}

