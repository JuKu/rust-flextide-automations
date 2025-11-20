//! GitLab API Integration
//! 
//! Provides a client for interacting with the GitLab API, including projects,
//! issues, merge requests, and other GitLab services.

mod client;
mod error;
mod types;

pub use client::GitLabClient;
pub use error::GitLabError;
pub use types::*;

