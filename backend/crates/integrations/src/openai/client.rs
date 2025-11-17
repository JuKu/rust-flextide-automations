//! OpenAI API Client
//! 
//! A client for making requests to the OpenAI API.

use crate::openai::error::OpenAIError;
use crate::openai::types::*;
use reqwest::Client;
use tracing::{debug, error, info};

const OPENAI_API_BASE: &str = "https://api.openai.com/v1";

/// Client for interacting with the OpenAI API
pub struct OpenAIClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl OpenAIClient {
    /// Create a new OpenAI client with the provided API key
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: OPENAI_API_BASE.to_string(),
        }
    }

    /// Create a new OpenAI client with a custom base URL (useful for proxies or alternative endpoints)
    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url,
        }
    }

    /// Send a chat completion request to the OpenAI API
    pub async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, OpenAIError> {
        let url = format!("{}/chat/completions", self.base_url);

        debug!("Sending chat completion request to OpenAI: model={}", request.model);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("OpenAI API error: status={}, body={}", status, error_text);

            return match status.as_u16() {
                401 => Err(OpenAIError::InvalidApiKey),
                429 => Err(OpenAIError::RateLimitExceeded),
                _ => Err(OpenAIError::ApiError(format!(
                    "HTTP {}: {}",
                    status, error_text
                ))),
            };
        }

        let completion: ChatCompletionResponse = response.json().await?;
        
        info!(
            "Chat completion successful: model={}, tokens={}",
            completion.model, completion.usage.total_tokens
        );

        Ok(completion)
    }
}

