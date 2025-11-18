//! User management module
//! 
//! Provides functionality for user management, password hashing, and validation.

mod database;
mod password;
mod validation;

pub use database::{
    ensure_default_admin_user, get_user_by_email, has_any_users, user_belongs_to_organization,
    user_has_permission, UserDatabaseError,
};
pub use password::{hash_password, verify_password, PasswordError};
pub use validation::{validate_password, validate_email, PasswordValidationError, EmailValidationError};

use thiserror::Error;

/// Error type for user creation
#[derive(Debug, Error)]
pub enum UserCreationError {
    #[error("Email validation failed: {0}")]
    EmailValidation(#[from] EmailValidationError),
    
    #[error("Password validation failed: {0}")]
    PasswordValidation(#[from] PasswordValidationError),
    
    #[error("Password hashing failed: {0}")]
    PasswordHashing(#[from] PasswordError),
}

/// User data structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub uuid: String,
    pub email: String,
    pub password_hash: String,
    pub salt: Option<String>,
    pub prename: String,
    pub lastname: Option<String>,
    pub mail_verified: bool,
    pub activated: bool,
}

/// User creation request
#[derive(Debug, serde::Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub prename: String,
    pub lastname: Option<String>,
}

/// User update request
#[derive(Debug, serde::Deserialize)]
pub struct UpdateUserRequest {
    pub prename: Option<String>,
    pub lastname: Option<String>,
    pub mail_verified: Option<bool>,
    pub activated: Option<bool>,
}

impl User {
    /// Create a new user from a creation request
    pub fn from_request(request: CreateUserRequest) -> Result<(Self, String), UserCreationError> {
        // Validate email
        validation::validate_email(&request.email)
            .map_err(UserCreationError::EmailValidation)?;
        
        // Validate password
        validation::validate_password(&request.password)
            .map_err(UserCreationError::PasswordValidation)?;
        
        // Hash password
        let password_hash = password::hash_password(&request.password)
            .map_err(UserCreationError::PasswordHashing)?;
        
        // Generate UUID
        let uuid = uuid::Uuid::new_v4().to_string();
        
        let password_hash_clone = password_hash.clone();
        Ok((
            Self {
                uuid,
                email: request.email,
                password_hash,
                salt: None, // Argon2 includes salt in hash string
                prename: request.prename,
                lastname: request.lastname,
                mail_verified: false,
                activated: true,
            },
            password_hash_clone,
        ))
    }
    
    /// Verify a password against this user's password hash
    pub fn verify_password(&self, password: &str) -> Result<bool, PasswordError> {
        password::verify_password(password, &self.password_hash)
    }
    
    /// Check if user account is active
    pub fn is_active(&self) -> bool {
        self.activated && self.mail_verified
    }
}

