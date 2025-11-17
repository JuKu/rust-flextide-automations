//! Flextide Integrations Library
//! 
//! Provides integration clients for external APIs and services.
//! This crate contains reusable client implementations for various third-party services
//! that can be used by nodes in the workflow automation platform.

pub mod openai;

pub use openai::OpenAIClient;

