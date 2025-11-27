//! Page Summary Generation Module
//!
//! Provides an interface for generating short summaries of documentation pages using AI.
//! Supports multiple AI providers through a trait-based architecture.
//!
//! # Example
//! ```rust,no_run
//! use flextide_modules_docs::summary::{OpenAIPageSummaryGenerator, PageSummaryGenerator};
//! use flextide_modules_docs::page::{DocsPage, DocsPageVersion};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let generator = OpenAIPageSummaryGenerator::new(
//!     "your-api-key".to_string(),
//!     "gpt-4o-mini".to_string()
//! );
//!
//! // Assuming you have a page and version
//! // let summary = generator.generate_summary(&page, &version).await?;
//! # Ok(())
//! # }
//! ```

mod claude;
mod gemini;
mod openai;

pub use claude::ClaudePageSummaryGenerator;
pub use gemini::GeminiPageSummaryGenerator;
pub use openai::OpenAIPageSummaryGenerator;

use async_trait::async_trait;
use crate::page::{DocsPage, DocsPageVersion};
use thiserror::Error;

/// Error type for page summary generation
#[derive(Debug, Error)]
pub enum PageSummaryError {
    #[error("AI provider error: {0}")]
    ProviderError(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Content too long to summarize")]
    ContentTooLong,

    #[error("No content available to summarize")]
    NoContent,
}

/// Trait for generating page summaries using AI
///
/// This trait allows different AI providers (OpenAI, Claude, Gemini, etc.)
/// to be used interchangeably for generating short summaries of documentation pages.
#[async_trait]
pub trait PageSummaryGenerator: Send + Sync {
    /// Generate a short summary for a documentation page
    ///
    /// # Arguments
    /// * `page` - The documentation page to summarize
    /// * `version` - The latest version of the page containing the content to summarize
    ///
    /// # Returns
    /// Returns a `Result<String, PageSummaryError>` containing the generated summary
    ///
    /// # Errors
    /// Returns `PageSummaryError` if:
    /// - The AI provider request fails
    /// - The content is too long or empty
    /// - Authentication or rate limiting issues occur
    /// - Network errors occur
    async fn generate_summary(
        &self,
        page: &DocsPage,
        version: &DocsPageVersion,
    ) -> Result<String, PageSummaryError>;
}

