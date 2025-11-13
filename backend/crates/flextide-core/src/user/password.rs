//! Password hashing and verification using Argon2

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

/// Error type for password operations
#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("Password hashing failed: {0}")]
    HashingFailed(String),
}

/// Hash a password using Argon2
/// 
/// Returns the complete hash string in the format:
/// `$argon2id$v=19$m=19456,t=2,p=1$salt$hash`
/// 
/// The salt is included in the hash string, so you don't need to store it separately.
/// 
/// # Example
/// ```
/// use flextide_core::user::hash_password;
/// 
/// let hash = hash_password("my_secure_password")?;
/// // Store hash in database
/// ```
pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| PasswordError::HashingFailed(e.to_string()))?;
    Ok(password_hash.to_string())
}

/// Verify a password against an Argon2 hash
/// 
/// # Arguments
/// * `password` - The plain text password to verify
/// * `hash` - The Argon2 hash string (includes salt and parameters)
/// 
/// # Returns
/// * `Ok(true)` if password matches
/// * `Ok(false)` if password doesn't match
/// * `Err` if hash format is invalid
/// 
/// # Example
/// ```
/// use flextide_core::user::{hash_password, verify_password};
/// 
/// let hash = hash_password("my_password")?;
/// assert!(verify_password("my_password", &hash)?);
/// assert!(!verify_password("wrong_password", &hash)?);
/// ```
pub fn verify_password(password: &str, hash: &str) -> Result<bool, PasswordError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| PasswordError::HashingFailed(e.to_string()))?;
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();
        
        // Verify same password works
        assert!(verify_password(password, &hash).unwrap());
        
        // Verify wrong password fails
        assert!(!verify_password("wrong_password", &hash).unwrap());
        
        // Verify hash is different each time (due to salt)
        let hash2 = hash_password(password).unwrap();
        assert_ne!(hash, hash2);
        
        // But both should verify the same password
        assert!(verify_password(password, &hash).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }
    
    #[test]
    fn test_hash_format() {
        let hash = hash_password("test").unwrap();
        // Argon2 hash should start with $argon2
        assert!(hash.starts_with("$argon2"));
    }
}

