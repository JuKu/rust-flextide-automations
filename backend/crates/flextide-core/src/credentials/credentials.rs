//! Credentials Manager
//!
//! Handles encryption and decryption of credentials using AES-256-GCM.
//! The master key is stored in the CREDENTIALS_MASTER_KEY environment variable
//! and is never exposed outside this module.

use crate::credentials::error::CredentialsError;
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use hex;
use serde_json::Value;
use std::sync::Arc;

/// Master encryption key (32 bytes = 256 bits for AES-256)
type MasterKey = [u8; 32];

/// Credentials Manager
///
/// Handles encryption and decryption of credentials using AES-256-GCM.
/// The master key is loaded from the CREDENTIALS_MASTER_KEY environment variable
/// and is kept private within this struct.
pub struct CredentialsManager {
    /// Master encryption key (private, never exposed)
    master_key: Arc<MasterKey>,
}

impl CredentialsManager {
    /// Create a new CredentialsManager by loading the master key from environment
    ///
    /// # Errors
    /// Returns `CredentialsError` if:
    /// - CREDENTIALS_MASTER_KEY environment variable is not set
    /// - Master key is not in valid hex format (64 hex characters = 32 bytes)
    pub fn new() -> Result<Self, CredentialsError> {
        let master_key_str = std::env::var("CREDENTIALS_MASTER_KEY")
            .map_err(|_| CredentialsError::MasterKeyNotFound)?;

        // Master key must be 64 hex characters (32 bytes)
        if master_key_str.len() != 64 {
            return Err(CredentialsError::InvalidMasterKeyFormat);
        }

        let master_key = hex::decode(master_key_str)
            .map_err(|_| CredentialsError::InvalidMasterKeyFormat)?;

        if master_key.len() != 32 {
            return Err(CredentialsError::InvalidMasterKeyFormat);
        }

        let master_key_array: MasterKey = master_key
            .try_into()
            .map_err(|_| CredentialsError::InvalidMasterKeyFormat)?;

        Ok(Self {
            master_key: Arc::new(master_key_array),
        })
    }

    /// Encrypt credential data
    ///
    /// # Arguments
    /// * `data` - JSON value containing credential data to encrypt
    ///
    /// # Returns
    /// Encrypted data as bytes (nonce + ciphertext)
    ///
    /// # Errors
    /// Returns `CredentialsError` if encryption fails
    pub fn encrypt(&self, data: &Value) -> Result<Vec<u8>, CredentialsError> {
        // Serialize JSON to bytes
        let plaintext = serde_json::to_vec(data)
            .map_err(|e| CredentialsError::Serialization(e))?;

        // Create cipher with master key (dereference Arc to get slice)
        let cipher = Aes256Gcm::new_from_slice(self.master_key.as_ref())
            .map_err(|e| CredentialsError::Encryption(format!("Failed to create cipher: {}", e)))?;

        // Generate unique nonce (12 bytes for GCM)
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        // Encrypt data
        let ciphertext = cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| CredentialsError::Encryption(format!("Encryption failed: {}", e)))?;

        // Prepend nonce to ciphertext for storage
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt credential data
    ///
    /// # Arguments
    /// * `encrypted_data` - Encrypted data (nonce + ciphertext)
    ///
    /// # Returns
    /// Decrypted JSON value
    ///
    /// # Errors
    /// Returns `CredentialsError` if decryption fails
    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<Value, CredentialsError> {
        // Extract nonce (first 12 bytes) and ciphertext (rest)
        if encrypted_data.len() < 12 {
            return Err(CredentialsError::Decryption(
                "Encrypted data too short".to_string(),
            ));
        }

        // Extract nonce bytes (first 12 bytes)
        let nonce_bytes = &encrypted_data[..12];
        // Convert slice to fixed-size array to avoid deprecated from_slice
        let nonce_array: [u8; 12] = nonce_bytes
            .try_into()
            .map_err(|_| CredentialsError::Decryption("Invalid nonce length".to_string()))?;
        // Construct Nonce from array (avoids deprecated from_slice)
        // Nonce implements From<[u8; 12]> via GenericArray
        let nonce = Nonce::from(nonce_array);
        let ciphertext = &encrypted_data[12..];

        // Create cipher with master key (dereference Arc to get slice)
        let cipher = Aes256Gcm::new_from_slice(self.master_key.as_ref())
            .map_err(|e| CredentialsError::Decryption(format!("Failed to create cipher: {}", e)))?;

        // Decrypt data
        let plaintext = cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|e| CredentialsError::Decryption(format!("Decryption failed: {}", e)))?;

        // Deserialize JSON from bytes
        let value: Value = serde_json::from_slice(&plaintext)
            .map_err(|e| CredentialsError::Decryption(format!("Invalid JSON: {}", e)))?;

        Ok(value)
    }

}

impl Default for CredentialsManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize CredentialsManager - CREDENTIALS_MASTER_KEY must be set")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_manager() -> CredentialsManager {
        // Generate a test master key
        let test_key = hex::encode([0u8; 32]);
        std::env::set_var("CREDENTIALS_MASTER_KEY", test_key);
        CredentialsManager::new().unwrap()
    }

    #[test]
    fn test_encrypt_decrypt() {
        let manager = create_test_manager();
        let original_data = json!({
            "api_key": "test-key-123",
            "base_url": "https://api.example.com"
        });

        // Encrypt
        let encrypted = manager.encrypt(&original_data).unwrap();
        assert!(!encrypted.is_empty());
        assert!(encrypted.len() > 12); // Should have nonce + ciphertext

        // Decrypt
        let decrypted = manager.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, original_data);
    }

    #[test]
    fn test_encrypt_decrypt_different_data() {
        let manager = create_test_manager();
        let data1 = json!({"key": "value1"});
        let data2 = json!({"key": "value2"});

        let encrypted1 = manager.encrypt(&data1).unwrap();
        let encrypted2 = manager.encrypt(&data2).unwrap();

        // Encrypted data should be different (due to unique nonce)
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt correctly
        assert_eq!(manager.decrypt(&encrypted1).unwrap(), data1);
        assert_eq!(manager.decrypt(&encrypted2).unwrap(), data2);
    }

    #[test]
    fn test_invalid_encrypted_data() {
        let manager = create_test_manager();
        let invalid_data = vec![0u8; 10]; // Too short

        let result = manager.decrypt(&invalid_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_master_key_not_found() {
        std::env::remove_var("CREDENTIALS_MASTER_KEY");
        let result = CredentialsManager::new();
        assert!(result.is_err());
        match result.unwrap_err() {
            CredentialsError::MasterKeyNotFound => {}
            _ => panic!("Expected MasterKeyNotFound error"),
        }
    }

    #[test]
    fn test_invalid_master_key_format_short() {
        std::env::set_var("CREDENTIALS_MASTER_KEY", "short");
        let result = CredentialsManager::new();
        assert!(result.is_err());
        match result.unwrap_err() {
            CredentialsError::InvalidMasterKeyFormat => {}
            _ => panic!("Expected InvalidMasterKeyFormat error"),
        }
    }

    #[test]
    fn test_invalid_master_key_format_invalid_hex() {
        std::env::set_var("CREDENTIALS_MASTER_KEY", "x".repeat(64));
        let result = CredentialsManager::new();
        assert!(result.is_err());
        match result.unwrap_err() {
            CredentialsError::InvalidMasterKeyFormat => {}
            _ => panic!("Expected InvalidMasterKeyFormat error"),
        }
    }

    #[test]
    fn test_encrypt_decrypt_complex_json() {
        let manager = create_test_manager();
        let original_data = json!({
            "api_key": "test-key-123",
            "base_url": "https://api.example.com",
            "timeout": 30,
            "retries": 3,
            "headers": {
                "Authorization": "Bearer token",
                "Content-Type": "application/json"
            },
            "endpoints": ["/users", "/posts", "/comments"],
            "enabled": true,
            "metadata": null
        });

        let encrypted = manager.encrypt(&original_data).unwrap();
        let decrypted = manager.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, original_data);
    }

    #[test]
    fn test_encrypt_decrypt_empty_object() {
        let manager = create_test_manager();
        let original_data = json!({});

        let encrypted = manager.encrypt(&original_data).unwrap();
        let decrypted = manager.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, original_data);
    }

    #[test]
    fn test_encrypt_decrypt_large_data() {
        let manager = create_test_manager();
        let large_string = "x".repeat(10000);
        let original_data = json!({
            "large_data": large_string,
            "other_field": "value"
        });

        let encrypted = manager.encrypt(&original_data).unwrap();
        let decrypted = manager.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, original_data);
    }

    #[test]
    fn test_decrypt_corrupted_data() {
        let manager = create_test_manager();
        let original_data = json!({"key": "value"});
        let mut encrypted = manager.encrypt(&original_data).unwrap();
        
        // Corrupt the encrypted data
        encrypted[20] ^= 0xFF;
        
        let result = manager.decrypt(&encrypted);
        assert!(result.is_err());
        match result.unwrap_err() {
            CredentialsError::Decryption(_) => {}
            _ => panic!("Expected Decryption error"),
        }
    }

    #[test]
    fn test_decrypt_wrong_key() {
        // Create manager with one key
        let test_key1 = hex::encode([0u8; 32]);
        std::env::set_var("CREDENTIALS_MASTER_KEY", test_key1);
        let manager1 = CredentialsManager::new().unwrap();
        
        let original_data = json!({"key": "value"});
        let encrypted = manager1.encrypt(&original_data).unwrap();
        
        // Create manager with different key
        let test_key2 = hex::encode([1u8; 32]);
        std::env::set_var("CREDENTIALS_MASTER_KEY", test_key2);
        let manager2 = CredentialsManager::new().unwrap();
        
        // Should fail to decrypt with wrong key
        let result = manager2.decrypt(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_produces_different_output() {
        let manager = create_test_manager();
        let data = json!({"key": "value"});
        
        // Encrypt same data multiple times
        let encrypted1 = manager.encrypt(&data).unwrap();
        let encrypted2 = manager.encrypt(&data).unwrap();
        let encrypted3 = manager.encrypt(&data).unwrap();
        
        // All should be different due to unique nonces
        assert_ne!(encrypted1, encrypted2);
        assert_ne!(encrypted2, encrypted3);
        assert_ne!(encrypted1, encrypted3);
        
        // But all should decrypt to the same value
        assert_eq!(manager.decrypt(&encrypted1).unwrap(), data);
        assert_eq!(manager.decrypt(&encrypted2).unwrap(), data);
        assert_eq!(manager.decrypt(&encrypted3).unwrap(), data);
    }

    #[test]
    fn test_encrypt_decrypt_special_characters() {
        let manager = create_test_manager();
        let original_data = json!({
            "password": "p@ssw0rd!@#$%^&*()",
            "token": "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
            "url": "https://example.com/path?param=value&other=test"
        });

        let encrypted = manager.encrypt(&original_data).unwrap();
        let decrypted = manager.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, original_data);
    }
}

