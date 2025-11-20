# Secure Credential Storage Research

## Overview

This document outlines best practices for securely storing credentials (API keys, passwords, tokens) in databases for the Flextide platform.

## Key Principles

### 1. Encryption at Rest

Credentials must be encrypted before being stored in the database. The encryption should:
- Use strong, industry-standard algorithms (AES-256-GCM recommended)
- Use a master key stored separately from the database
- Support key rotation without data loss
- Never store encryption keys in the database or code

### 2. Master Key Management

The master encryption key should:
- Be stored in environment variables (never in code or version control)
- Be at least 256 bits (32 bytes) for AES-256
- Be generated using cryptographically secure random number generators
- Be rotated periodically (with re-encryption of existing data)
- Be backed up securely (separate from database backups)

### 3. Encryption Algorithm: AES-256-GCM

**AES-256-GCM (Galois/Counter Mode)** is recommended because:
- **Authenticated Encryption**: Provides both confidentiality and authenticity
- **Industry Standard**: Widely used and well-vetted
- **Performance**: Hardware-accelerated on modern CPUs
- **Nonce-based**: Each encryption uses a unique nonce, preventing replay attacks
- **Integrity**: Detects tampering automatically

**Key Size**: 256 bits (32 bytes) for AES-256

**Nonce/IV**: 96 bits (12 bytes) for GCM mode - must be unique for each encryption

### 4. Database Storage Strategy

#### Encrypted Data Storage
- Store encrypted credentials as binary data (BLOB) or base64-encoded text
- Store nonce/IV alongside encrypted data (or derive from record UUID)
- Store encryption key version to support key rotation
- Optional salt field for additional security (key derivation, per-credential keys, etc.)
- Never store plaintext credentials, even temporarily

#### Metadata Storage
- Store unencrypted metadata (name, type, organization_uuid, timestamps) for querying
- Use metadata for filtering and listing without decrypting
- Keep metadata minimal to reduce information leakage

### 5. Access Control

- **Organization Isolation**: Credentials belong to organizations, enforce strict access control
- **Permission Checks**: Require explicit permissions to create/read/update/delete credentials
- **Audit Logging**: Log all credential access for security monitoring
- **Least Privilege**: Only decrypt credentials when absolutely necessary

### 6. Key Rotation

Key rotation is critical for long-term security:
- Generate new master key periodically (e.g., annually)
- Track encryption key version for each credential (`encryption_key_version` column)
- Re-encrypt all credentials with new key and update version number
- Keep old key temporarily for decryption during transition
- Invalidate old key after migration complete
- Query credentials by `encryption_key_version` to identify which need re-encryption

**Implementation:**
- Store `encryption_key_version` (integer, default: 1) with each credential
- When rotating keys, increment version number
- Support multiple key versions during transition period
- Decrypt using the key version specified in the credential record

## Implementation Approach

### Rust Implementation

Use the `aes-gcm` crate for AES-256-GCM encryption:

```rust
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};

// Master key: 32 bytes (256 bits) for AES-256
type MasterKey = [u8; 32];

// Encrypt credential data
fn encrypt(plaintext: &[u8], key: &MasterKey) -> Result<Vec<u8>, Error> {
    let cipher = Aes256Gcm::new_from_slice(key)?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext)?;
    
    // Prepend nonce to ciphertext for storage
    let mut result = nonce.to_vec();
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

// Decrypt credential data
fn decrypt(ciphertext: &[u8], key: &MasterKey) -> Result<Vec<u8>, Error> {
    let cipher = Aes256Gcm::new_from_slice(key)?;
    
    // Extract nonce (first 12 bytes)
    let nonce = Nonce::from_slice(&ciphertext[..12]);
    let encrypted_data = &ciphertext[12..];
    
    let plaintext = cipher.decrypt(nonce, encrypted_data)?;
    Ok(plaintext)
}
```

### Database Schema

```sql
CREATE TABLE IF NOT EXISTS credentials (
    uuid CHAR(36) NOT NULL PRIMARY KEY,
    organization_uuid CHAR(36) NOT NULL,
    name VARCHAR(255) NOT NULL,
    type VARCHAR(255) NOT NULL,
    encrypted_data BLOB NOT NULL,  -- Encrypted JSON credential data
    encryption_key_version INTEGER NOT NULL DEFAULT 1,  -- Version of encryption key used (for key rotation)
    salt VARCHAR(255) NULL,  -- Optional salt for additional security layer
    creator_user_uuid CHAR(36) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NULL,
    
    FOREIGN KEY (organization_uuid) REFERENCES organizations(uuid) ON DELETE CASCADE,
    FOREIGN KEY (creator_user_uuid) REFERENCES users(uuid) ON DELETE RESTRICT,
    
    INDEX idx_credentials_organization (organization_uuid),
    INDEX idx_credentials_type (type),
    INDEX idx_credentials_name (name)
);
```

**Storage Format**:
- `encrypted_data`: Binary blob containing nonce (12 bytes) + encrypted JSON credential data
- `encryption_key_version`: Integer indicating which master key version was used (default: 1)
- `salt`: Optional VARCHAR(255) for additional security (can be used for key derivation, per-credential keys, etc.)
- Store as BLOB in MySQL/PostgreSQL, or base64-encode if using TEXT column

**Key Rotation Support**:
- The `encryption_key_version` field allows tracking which master key was used to encrypt each credential
- During key rotation, credentials can be identified by version and re-encrypted with the new key
- Multiple key versions can coexist during the transition period

### Master Key Generation

Generate a secure master key:

```bash
# Using OpenSSL
openssl rand -hex 32

# Using Python
python3 -c "import secrets; print(secrets.token_hex(32))"

# Using Rust (for key generation utility)
use rand::RngCore;
let mut key = [0u8; 32];
rand::thread_rng().fill_bytes(&mut key);
let hex_key = hex::encode(key);
```

**Storage**: Store in environment variable `CREDENTIALS_MASTER_KEY`

## Security Considerations

### 1. Memory Safety
- Clear sensitive data from memory after use when possible
- Use Rust's ownership system to prevent accidental leaks
- Avoid logging credential data (even encrypted)

### 2. Key Exposure Risks
- Never commit master key to version control
- Use `.env` files (gitignored) for local development
- Use secure secret management in production (AWS Secrets Manager, HashiCorp Vault, etc.)
- Rotate keys if exposure is suspected

### 3. Database Security
- Use encrypted database connections (TLS/SSL)
- Restrict database access to application servers only
- Enable database-level encryption at rest (if available)
- Regular database backups (encrypted)

### 4. Application Security
- Validate all inputs before encryption
- Use parameterized queries (prevent SQL injection)
- Implement rate limiting on credential operations
- Log access attempts (without logging credential data)

### 5. Operational Security
- Limit who can access the master key
- Use separate keys for different environments (dev/staging/prod)
- Implement key rotation procedures
- Monitor for unauthorized access

## Comparison with Alternatives

### Alternative 1: Hashing (Not Suitable)
- **Problem**: Credentials must be retrievable (not one-way)
- **Use Case**: Only for passwords (use Argon2/bcrypt)

### Alternative 2: Database-Level Encryption
- **Pros**: Transparent, automatic
- **Cons**: Keys managed by database, less control, vendor-specific
- **Verdict**: Good supplement, but not replacement for application-level encryption

### Alternative 3: External Secret Management (HashiCorp Vault, AWS Secrets Manager)
- **Pros**: Professional key management, rotation, audit
- **Cons**: Additional dependency, network calls, cost
- **Verdict**: Consider for production, but application-level encryption still needed

### Alternative 4: Key Derivation (PBKDF2, Argon2)
- **Use Case**: Deriving keys from passwords (not for master keys)
- **Verdict**: Use for user-provided passphrases, not for system master keys

## Best Practices Summary

1. ✅ **Use AES-256-GCM** for encryption
2. ✅ **Store master key in environment variable** (never in code)
3. ✅ **Generate unique nonce** for each encryption
4. ✅ **Store nonce with ciphertext** (or derive deterministically)
5. ✅ **Encrypt at application level** (before database storage)
6. ✅ **Use BLOB or base64** for encrypted data storage
7. ✅ **Keep metadata unencrypted** for querying
8. ✅ **Implement key rotation** procedures
9. ✅ **Log access** (without logging credentials)
10. ✅ **Enforce access control** at application level

## References

- [OWASP Cryptographic Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html)
- [NIST Guidelines for Key Management](https://csrc.nist.gov/publications/detail/sp/800-57-part-1/rev-5/final)
- [AES-GCM Specification](https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38d.pdf)
- [RustCrypto AES-GCM](https://github.com/RustCrypto/AEADs/tree/master/aes-gcm)

