//! Type definitions for Chroma API requests and responses

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Collection metadata
pub type CollectionMetadata = HashMap<String, serde_json::Value>;

/// Document metadata
pub type DocumentMetadata = HashMap<String, serde_json::Value>;

/// Request to create a new collection
#[derive(Debug, Clone, Serialize)]
pub struct CreateCollectionRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<CollectionMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_function: Option<String>,
}

/// Collection information
#[derive(Debug, Clone, Deserialize)]
pub struct Collection {
    pub name: String,
    pub id: String,
    #[serde(default)]
    pub metadata: CollectionMetadata,
}

/// Request to add documents to a collection
#[derive(Debug, Clone, Serialize)]
pub struct AddDocumentsRequest {
    pub ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documents: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadatas: Option<Vec<DocumentMetadata>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeddings: Option<Vec<Vec<f32>>>,
}

/// Request to update documents in a collection
#[derive(Debug, Clone, Serialize)]
pub struct UpdateDocumentsRequest {
    pub ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documents: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadatas: Option<Vec<DocumentMetadata>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeddings: Option<Vec<Vec<f32>>>,
}

/// Metadata filter for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetadataFilter {
    /// Simple key-value equality
    Equals(HashMap<String, serde_json::Value>),
    /// Complex filter with operators
    Complex(serde_json::Value),
}

/// Document content filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentFilter {
    #[serde(flatten)]
    pub filter: HashMap<String, serde_json::Value>,
}

/// Request to delete documents from a collection
#[derive(Debug, Clone, Serialize)]
pub struct DeleteDocumentsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<String>>,
    #[serde(rename = "where", skip_serializing_if = "Option::is_none")]
    pub where_clause: Option<MetadataFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub where_document: Option<DocumentFilter>,
}

/// Request to query a collection
#[derive(Debug, Clone, Serialize)]
pub struct QueryRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_texts: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_embeddings: Option<Vec<Vec<f32>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n_results: Option<usize>,
    #[serde(rename = "where", skip_serializing_if = "Option::is_none")]
    pub where_clause: Option<MetadataFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub where_document: Option<DocumentFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
}

/// Request to get documents from a collection
#[derive(Debug, Clone, Serialize)]
pub struct GetDocumentsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<String>>,
    #[serde(rename = "where", skip_serializing_if = "Option::is_none")]
    pub where_clause: Option<MetadataFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub where_document: Option<DocumentFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
}

/// Query result containing matched documents
#[derive(Debug, Clone, Deserialize)]
pub struct QueryResult {
    pub ids: Vec<Vec<String>>,
    #[serde(default)]
    pub documents: Vec<Vec<Option<String>>>,
    #[serde(default)]
    pub metadatas: Vec<Vec<Option<DocumentMetadata>>>,
    #[serde(default)]
    pub distances: Vec<Vec<f32>>,
    #[serde(default)]
    pub embeddings: Vec<Vec<Vec<f32>>>,
}

/// Get documents result
#[derive(Debug, Clone, Deserialize)]
pub struct GetDocumentsResult {
    pub ids: Vec<String>,
    #[serde(default)]
    pub documents: Vec<Option<String>>,
    #[serde(default)]
    pub metadatas: Vec<Option<DocumentMetadata>>,
    #[serde(default)]
    pub embeddings: Vec<Vec<f32>>,
}

/// Count result
#[derive(Debug, Clone, Deserialize)]
pub struct CountResult {
    pub count: usize,
}

/// Peek result (sample documents)
#[derive(Debug, Clone, Deserialize)]
pub struct PeekResult {
    pub ids: Vec<String>,
    #[serde(default)]
    pub documents: Vec<Option<String>>,
    #[serde(default)]
    pub metadatas: Vec<Option<DocumentMetadata>>,
}

