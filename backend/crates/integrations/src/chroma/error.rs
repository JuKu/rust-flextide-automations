//! Error types for Chroma API integration

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChromaError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Collection not found: {0}")]
    CollectionNotFound(String),

    #[error("Collection already exists: {0}")]
    CollectionExists(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    #[error("Invalid embedding dimensions: expected {expected}, got {actual}")]
    InvalidEmbeddingDimensions { expected: usize, actual: usize },

    #[error("Missing required field: {0}")]
    MissingField(String),
}

