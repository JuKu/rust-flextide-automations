//! Credentials Management Module
//!
//! Provides secure storage and retrieval of credentials (API keys, tokens, etc.)
//! for external services. Credentials are encrypted using AES-256-GCM before
//! being stored in the database.

mod credentials;
mod database;
mod error;

pub use credentials::CredentialsManager;
pub use database::*;
pub use error::CredentialsError;

