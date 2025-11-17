//! Error types for OpenAI API integration

use thiserror::Error;

#[derive(Debug, Error)]
pub enum OpenAIError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
}

