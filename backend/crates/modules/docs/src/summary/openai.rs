//! OpenAI implementation of PageSummaryGenerator
//!
//! Uses OpenAI's chat completion API to generate page summaries.

use async_trait::async_trait;
use crate::page::{DocsPage, DocsPageVersion};
use crate::summary::{PageSummaryError, PageSummaryGenerator};
use integrations::openai::{ChatCompletionRequest, ChatMessage, MessageRole, OpenAIClient};
use tracing::{debug, error, warn};

/// OpenAI-based page summary generator
pub struct OpenAIPageSummaryGenerator {
    client: OpenAIClient,
    model: String,
    max_summary_length: Option<usize>,
}

impl OpenAIPageSummaryGenerator {
    /// Create a new OpenAI page summary generator
    ///
    /// # Arguments
    /// * `api_key` - OpenAI API key
    /// * `model` - Model to use for summarization (e.g., "gpt-4o-mini", "gpt-4o")
    ///
    /// # Returns
    /// Returns a new `OpenAIPageSummaryGenerator` instance
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: OpenAIClient::new(api_key),
            model,
            max_summary_length: Some(200), // Default to 200 characters
        }
    }

    /// Create a new OpenAI page summary generator with a custom base URL
    ///
    /// # Arguments
    /// * `api_key` - OpenAI API key
    /// * `base_url` - Custom base URL (useful for proxies or alternative endpoints)
    /// * `model` - Model to use for summarization
    ///
    /// # Returns
    /// Returns a new `OpenAIPageSummaryGenerator` instance
    pub fn with_base_url(api_key: String, base_url: String, model: String) -> Self {
        Self {
            client: OpenAIClient::with_base_url(api_key, base_url),
            model,
            max_summary_length: Some(200),
        }
    }

    /// Set the maximum summary length in characters
    ///
    /// # Arguments
    /// * `length` - Maximum length in characters (None for no limit)
    pub fn with_max_summary_length(mut self, length: Option<usize>) -> Self {
        self.max_summary_length = length;
        self
    }

    /// Truncate content if it's too long for the model's context window
    ///
    /// OpenAI models have token limits. This function provides a rough estimate
    /// to truncate content (assuming ~4 characters per token).
    fn truncate_content(&self, content: &str, max_tokens: usize) -> String {
        // Rough estimate: 4 characters per token
        let max_chars = max_tokens * 4;
        if content.len() > max_chars {
            let truncated = content.chars().take(max_chars).collect::<String>();
            warn!(
                "Content truncated from {} to {} characters for summarization",
                content.len(),
                truncated.len()
            );
            truncated
        } else {
            content.to_string()
        }
    }
}

#[async_trait]
impl PageSummaryGenerator for OpenAIPageSummaryGenerator {
    async fn generate_summary(
        &self,
        page: &DocsPage,
        version: &DocsPageVersion,
    ) -> Result<String, PageSummaryError> {
        // Check if content is empty
        if version.content.trim().is_empty() {
            return Err(PageSummaryError::NoContent);
        }

        // Truncate content if necessary (most OpenAI models have ~128k token context)
        // Using a conservative 100k tokens to leave room for the prompt
        let content = self.truncate_content(&version.content, 100_000);

        // Build the prompt
        let system_prompt = "You are a documentation assistant. Generate a concise, informative summary of the following documentation page. The summary should be clear, professional, and capture the key points. Keep it brief and focused.";
        
        let user_prompt = format!(
            "Page Title: {}\n\nPage Content:\n{}\n\nGenerate a short summary (maximum {} characters):",
            page.title,
            content,
            self.max_summary_length.unwrap_or(500)
        );

        debug!(
            "Generating summary for page {} using OpenAI model {}",
            page.uuid, self.model
        );

        // Create the chat completion request
        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: MessageRole::System,
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: MessageRole::User,
                    content: user_prompt,
                },
            ],
            temperature: Some(0.3), // Lower temperature for more consistent summaries
            max_tokens: Some(150),  // Limit tokens to keep summaries concise
            stream: Some(false),
        };

        // Call OpenAI API
        let response = self.client.chat_completion(request).await
            .map_err(|e| {
                error!("OpenAI API error: {}", e);
                match e {
                    integrations::openai::OpenAIError::InvalidApiKey => {
                        PageSummaryError::AuthenticationFailed
                    }
                    integrations::openai::OpenAIError::RateLimitExceeded => {
                        PageSummaryError::RateLimitExceeded
                    }
                    integrations::openai::OpenAIError::ApiError(msg) => {
                        PageSummaryError::ProviderError(format!("OpenAI API error: {}", msg))
                    }
                    integrations::openai::OpenAIError::HttpError(http_err) => {
                        PageSummaryError::NetworkError(http_err.to_string())
                    }
                    integrations::openai::OpenAIError::SerializationError(serde_err) => {
                        PageSummaryError::ProviderError(format!("Serialization error: {}", serde_err))
                    }
                    integrations::openai::OpenAIError::InvalidResponse(msg) => {
                        PageSummaryError::ProviderError(format!("Invalid response: {}", msg))
                    }
                }
            })?;

        // Extract the summary from the response
        let summary = response
            .choices
            .first()
            .and_then(|choice| Some(choice.message.content.trim().to_string()))
            .ok_or_else(|| {
                error!("OpenAI response missing content");
                PageSummaryError::ProviderError("No content in OpenAI response".to_string())
            })?;

        // Truncate to max length if specified
        let summary = if let Some(max_len) = self.max_summary_length {
            if summary.len() > max_len {
                let truncated = summary.chars().take(max_len).collect::<String>();
                warn!(
                    "Summary truncated from {} to {} characters",
                    summary.len(),
                    truncated.len()
                );
                truncated
            } else {
                summary
            }
        } else {
            summary
        };

        debug!(
            "Successfully generated summary for page {} (length: {})",
            page.uuid,
            summary.len()
        );

        Ok(summary)
    }
}

