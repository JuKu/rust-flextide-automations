//! OpenAI API Integration
//! 
//! Provides a client for interacting with the OpenAI API, including chat completions,
//! embeddings, and other OpenAI services.

mod client;
mod error;
mod types;

pub use client::OpenAIClient;
pub use error::OpenAIError;
pub use types::*;

