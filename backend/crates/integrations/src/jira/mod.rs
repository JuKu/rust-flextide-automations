//! Jira API Integration
//! 
//! Provides a client for interacting with the Jira API, including issues,
//! projects, and other Jira services.

mod client;
mod error;
mod types;

pub use client::JiraClient;
pub use error::JiraError;
pub use types::*;

