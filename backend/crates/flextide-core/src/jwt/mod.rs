//! JWT token handling
//!
//! Provides JWT token structures and utilities for authentication.

use serde::{Deserialize, Serialize};

/// JWT Claims structure
///
/// Represents the claims contained in a JWT token for user authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// User's email address (subject)
    pub sub: String,
    /// User's UUID
    pub user_uuid: String,
    /// Token expiration timestamp
    pub exp: usize,
    /// Token issued at timestamp
    pub iat: usize,
    /// Whether the user is a server administrator
    pub is_server_admin: bool,
}

