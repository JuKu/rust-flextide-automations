//! GitHub API Integration
//! 
//! Provides a client for interacting with the GitHub API, including repositories,
//! issues, pull requests, and other GitHub services.

mod client;
mod error;
mod types;

pub use client::GitHubClient;
pub use error::GitHubError;
pub use types::*;

