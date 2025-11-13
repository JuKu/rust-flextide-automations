//! User input validation functions

use thiserror::Error;

/// Error type for validation operations
#[derive(Debug, Error, PartialEq)]
pub enum PasswordValidationError {
    #[error("Password must be at least 10 characters long")]
    TooShort,
    
    #[error("Password must be no more than 128 characters")]
    TooLong,
    
    #[error("Password is too common or easily guessable")]
    TooCommon,
}

/// Error type for email validation
#[derive(Debug, Error, PartialEq)]
pub enum EmailValidationError {
    #[error("Invalid email format")]
    InvalidFormat,
    
    #[error("Email is too long (maximum 255 characters)")]
    TooLong,
}

/// Validate password strength
/// 
/// Enforces:
/// - Minimum length: 10 characters
/// - Maximum length: 128 characters
/// - Rejects common weak passwords
/// 
/// # Example
/// ```
/// use flextide_core::user::validate_password;
/// 
/// assert!(validate_password("secure_password_123").is_ok());
/// assert!(validate_password("short").is_err());
/// ```
pub fn validate_password(password: &str) -> Result<(), PasswordValidationError> {
    // Minimum length: 10 characters
    if password.len() < 10 {
        return Err(PasswordValidationError::TooShort);
    }
    
    // Maximum length: 128 characters (prevent DoS)
    if password.len() > 128 {
        return Err(PasswordValidationError::TooLong);
    }
    
    // Check for common weak passwords
    let common_passwords = [
        "password", "12345678", "qwerty", "admin", "letmein",
        "welcome", "monkey", "1234567890", "password123",
    ];
    let password_lower = password.to_lowercase();
    if common_passwords.iter().any(|&p| password_lower.contains(p)) {
        return Err(PasswordValidationError::TooCommon);
    }
    
    Ok(())
}

/// Validate email format
/// 
/// Basic email validation - checks for:
/// - Contains @ symbol
/// - Has valid domain part
/// - Reasonable length (max 255 characters)
/// 
/// # Example
/// ```
/// use flextide_core::user::validate_email;
/// 
/// assert!(validate_email("user@example.com").is_ok());
/// assert!(validate_email("invalid").is_err());
/// ```
pub fn validate_email(email: &str) -> Result<(), EmailValidationError> {
    // Maximum length check (database VARCHAR(255) limit)
    if email.len() > 255 {
        return Err(EmailValidationError::TooLong);
    }
    
    // Basic format check
    if !email.contains('@') {
        return Err(EmailValidationError::InvalidFormat);
    }
    
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return Err(EmailValidationError::InvalidFormat);
    }
    
    let local = parts[0];
    let domain = parts[1];
    
    // Local part must not be empty
    if local.is_empty() {
        return Err(EmailValidationError::InvalidFormat);
    }
    
    // Domain must not be empty and contain at least one dot
    if domain.is_empty() || !domain.contains('.') {
        return Err(EmailValidationError::InvalidFormat);
    }
    
    // Basic character validation (no spaces, reasonable characters)
    if email.contains(' ') || email.contains('\n') || email.contains('\r') {
        return Err(EmailValidationError::InvalidFormat);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_validation() {
        // Valid passwords
        assert!(validate_password("secure_password_123").is_ok());
        assert!(validate_password("ThisIsAVeryLongPassword123!@#").is_ok());
        
        // Too short
        assert_eq!(
            validate_password("short"),
            Err(PasswordValidationError::TooShort)
        );
        
        // Too long
        let long_password = "a".repeat(129);
        assert_eq!(
            validate_password(&long_password),
            Err(PasswordValidationError::TooLong)
        );
        
        // Common password
        assert_eq!(
            validate_password("password123456"),
            Err(PasswordValidationError::TooCommon)
        );
    }
    
    #[test]
    fn test_email_validation() {
        // Valid emails
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("test.user+tag@example.co.uk").is_ok());
        
        // Invalid formats
        assert_eq!(
            validate_email("invalid"),
            Err(EmailValidationError::InvalidFormat)
        );
        assert_eq!(
            validate_email("@example.com"),
            Err(EmailValidationError::InvalidFormat)
        );
        assert_eq!(
            validate_email("user@"),
            Err(EmailValidationError::InvalidFormat)
        );
        assert_eq!(
            validate_email("user@nodomain"),
            Err(EmailValidationError::InvalidFormat)
        );
        assert_eq!(
            validate_email("user @example.com"),
            Err(EmailValidationError::InvalidFormat)
        );
        
        // Too long
        let long_email = format!("{}@example.com", "a".repeat(250));
        assert_eq!(
            validate_email(&long_email),
            Err(EmailValidationError::TooLong)
        );
    }
}

