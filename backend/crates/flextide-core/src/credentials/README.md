# Credentials Management Module

This module provides secure storage and retrieval of credentials (API keys, tokens, etc.) for external services. Credentials are encrypted using AES-256-GCM before being stored in the database.

## Overview

The credentials module consists of three main components:

1. **CredentialsManager** - Handles encryption and decryption of credential data
2. **Database Operations** - CRUD operations for managing credentials in the database
3. **Error Types** - Comprehensive error handling for all operations

## Security

- **Encryption**: All credential data is encrypted using AES-256-GCM before storage
- **Master Key**: Stored in `CREDENTIALS_MASTER_KEY` environment variable (never in code)
- **Access Control**: All operations require organization membership and specific permissions
- **Unique Nonces**: Each encryption uses a unique nonce, ensuring identical data produces different ciphertext

## Setup

### Environment Variable

Set the master encryption key in your environment:

```bash
# Generate a secure master key (64 hex characters = 32 bytes)
python -c "import secrets; print(secrets.token_hex(32))"
# Or: openssl rand -hex 32

# Set in .env file
CREDENTIALS_MASTER_KEY=your_generated_key_here
```

**Important**: Never commit the master key to version control. Use `.env` files (gitignored) for local development and secure secret management services for production.

## Usage

### Initializing CredentialsManager

```rust
use flextide_core::credentials::CredentialsManager;

// Create a new manager (loads master key from environment)
let manager = CredentialsManager::new()?;

// Or use Default (panics if master key not found)
let manager = CredentialsManager::default();
```

### CredentialsManager Methods

#### `new() -> Result<Self, CredentialsError>`

Creates a new `CredentialsManager` by loading the master key from the `CREDENTIALS_MASTER_KEY` environment variable.

**Returns:**
- `Ok(CredentialsManager)` - Successfully initialized manager
- `Err(CredentialsError::MasterKeyNotFound)` - Environment variable not set
- `Err(CredentialsError::InvalidMasterKeyFormat)` - Master key is not 64 hex characters

**Example:**
```rust
let manager = CredentialsManager::new()?;
```

#### `encrypt(data: &Value) -> Result<Vec<u8>, CredentialsError>`

Encrypts credential data (JSON) using AES-256-GCM.

**Arguments:**
- `data` - JSON value containing credential data to encrypt

**Returns:**
- `Ok(Vec<u8>)` - Encrypted data as bytes (nonce + ciphertext)
- `Err(CredentialsError::Serialization)` - Failed to serialize JSON
- `Err(CredentialsError::Encryption)` - Encryption failed

**Example:**
```rust
use serde_json::json;

let credential_data = json!({
    "api_key": "sk-1234567890abcdef",
    "base_url": "https://api.example.com"
});

let encrypted = manager.encrypt(&credential_data)?;
// encrypted contains: [12-byte nonce][encrypted JSON bytes]
```

#### `decrypt(encrypted_data: &[u8]) -> Result<Value, CredentialsError>`

Decrypts encrypted credential data back to JSON.

**Arguments:**
- `encrypted_data` - Encrypted data (nonce + ciphertext)

**Returns:**
- `Ok(Value)` - Decrypted JSON value
- `Err(CredentialsError::Decryption)` - Decryption failed or invalid format

**Example:**
```rust
let decrypted = manager.decrypt(&encrypted_data)?;
// decrypted is the original JSON value
```

## Database Operations

All database operations require:
1. User belongs to the organization
2. User has the appropriate permission

### Required Permissions

- `can_view_credentials` - List and view credentials
- `can_create_credentials` - Create new credentials
- `can_edit_credentials` - Update existing credentials
- `can_delete_credentials` - Delete credentials

### Data Structures

#### `CredentialMetadata`

Metadata about a credential (without encrypted data), used for listing:

```rust
pub struct CredentialMetadata {
    pub uuid: String,
    pub organization_uuid: String,
    pub name: String,
    pub credential_type: String,
    pub creator_user_uuid: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}
```

#### `Credential`

Full credential with decrypted data:

```rust
pub struct Credential {
    pub uuid: String,
    pub organization_uuid: String,
    pub name: String,
    pub credential_type: String,
    pub data: Value, // Decrypted JSON data
    pub creator_user_uuid: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}
```

### Database Methods

#### `list_credentials(pool, organization_uuid, user_uuid) -> Result<Vec<CredentialMetadata>, CredentialsError>`

Lists all credentials for an organization (metadata only, no encrypted data).

**Arguments:**
- `pool: &DatabasePool` - Database connection pool
- `organization_uuid: &str` - UUID of the organization
- `user_uuid: &str` - UUID of the user requesting the list

**Returns:**
- `Ok(Vec<CredentialMetadata>)` - List of credential metadata
- `Err(CredentialsError::UserNotInOrganization)` - User doesn't belong to organization
- `Err(CredentialsError::PermissionDenied)` - User lacks `can_view_credentials` permission
- `Err(CredentialsError::Database)` - Database operation failed

**Example:**
```rust
use flextide_core::credentials::list_credentials;

let credentials = list_credentials(
    &pool,
    "org-uuid-here",
    "user-uuid-here"
).await?;

for cred in credentials {
    println!("Credential: {} (type: {})", cred.name, cred.credential_type);
}
```

#### `create_credential(pool, manager, organization_uuid, user_uuid, name, credential_type, data) -> Result<String, CredentialsError>`

Creates a new credential with encrypted data.

**Arguments:**
- `pool: &DatabasePool` - Database connection pool
- `manager: &CredentialsManager` - Credentials manager for encryption
- `organization_uuid: &str` - UUID of the organization
- `user_uuid: &str` - UUID of the user creating the credential
- `name: &str` - Name of the credential (e.g., "Production Chroma API Key")
- `credential_type: &str` - Type of credential (e.g., "chroma", "openai", "github")
- `data: &Value` - JSON data to encrypt and store

**Returns:**
- `Ok(String)` - UUID of the created credential
- `Err(CredentialsError::UserNotInOrganization)` - User doesn't belong to organization
- `Err(CredentialsError::PermissionDenied)` - User lacks `can_create_credentials` permission
- `Err(CredentialsError::Encryption)` - Encryption failed
- `Err(CredentialsError::Database)` - Database operation failed

**Example:**
```rust
use flextide_core::credentials::{create_credential, CredentialsManager};
use serde_json::json;

let manager = CredentialsManager::new()?;

let credential_data = json!({
    "api_key": "sk-1234567890abcdef",
    "base_url": "https://api.example.com"
});

let credential_uuid = create_credential(
    &pool,
    &manager,
    "org-uuid-here",
    "user-uuid-here",
    "Production API Key",
    "openai",
    &credential_data
).await?;

println!("Created credential: {}", credential_uuid);
```

#### `get_credential(pool, manager, credential_uuid, organization_uuid, user_uuid) -> Result<Credential, CredentialsError>`

Gets a single credential by UUID with decrypted data.

**Arguments:**
- `pool: &DatabasePool` - Database connection pool
- `manager: &CredentialsManager` - Credentials manager for decryption
- `credential_uuid: &str` - UUID of the credential
- `organization_uuid: &str` - UUID of the organization (for access control)
- `user_uuid: &str` - UUID of the user requesting the credential

**Returns:**
- `Ok(Credential)` - Credential with decrypted data
- `Err(CredentialsError::CredentialNotFound)` - Credential doesn't exist
- `Err(CredentialsError::UserNotInOrganization)` - User doesn't belong to organization
- `Err(CredentialsError::PermissionDenied)` - User lacks `can_view_credentials` permission
- `Err(CredentialsError::Decryption)` - Decryption failed
- `Err(CredentialsError::Database)` - Database operation failed

**Example:**
```rust
use flextide_core::credentials::get_credential;

let credential = get_credential(
    &pool,
    &manager,
    "credential-uuid-here",
    "org-uuid-here",
    "user-uuid-here"
).await?;

println!("Credential name: {}", credential.name);
println!("Credential data: {}", credential.data);
```

#### `get_credentials(pool, manager, credential_uuids, organization_uuid, user_uuid) -> Result<Vec<Credential>, CredentialsError>`

Gets multiple credentials by UUIDs with decrypted data. This method is used by workflows to retrieve credentials.

**Arguments:**
- `pool: &DatabasePool` - Database connection pool
- `manager: &CredentialsManager` - Credentials manager for decryption
- `credential_uuids: &[String]` - Vector of credential UUIDs
- `organization_uuid: &str` - UUID of the organization (for access control)
- `user_uuid: &str` - UUID of the user requesting the credentials

**Returns:**
- `Ok(Vec<Credential>)` - Vector of credentials with decrypted data
- `Err(CredentialsError::UserNotInOrganization)` - User doesn't belong to organization
- `Err(CredentialsError::PermissionDenied)` - User lacks `can_view_credentials` permission
- `Err(CredentialsError::Decryption)` - Decryption failed for one or more credentials
- `Err(CredentialsError::Database)` - Database operation failed

**Note:** This function does not require individual credential permissions, but requires organization membership and general credential viewing permission.

**Example:**
```rust
use flextide_core::credentials::get_credentials;

let credential_uuids = vec![
    "credential-uuid-1".to_string(),
    "credential-uuid-2".to_string(),
];

let credentials = get_credentials(
    &pool,
    &manager,
    &credential_uuids,
    "org-uuid-here",
    "user-uuid-here"
).await?;

for cred in credentials {
    println!("Credential: {} - {}", cred.name, cred.credential_type);
    // Use cred.data to access decrypted credential information
}
```

#### `update_credential(pool, manager, credential_uuid, organization_uuid, user_uuid, name, data) -> Result<(), CredentialsError>`

Updates a credential (overwrites encrypted data). Optionally updates the name.

**Arguments:**
- `pool: &DatabasePool` - Database connection pool
- `manager: &CredentialsManager` - Credentials manager for encryption
- `credential_uuid: &str` - UUID of the credential to update
- `organization_uuid: &str` - UUID of the organization
- `user_uuid: &str` - UUID of the user updating the credential
- `name: Option<&str>` - New name (optional, None to keep existing name)
- `data: &Value` - New JSON data to encrypt and store

**Returns:**
- `Ok(())` - Success
- `Err(CredentialsError::CredentialNotFound)` - Credential doesn't exist
- `Err(CredentialsError::UserNotInOrganization)` - User doesn't belong to organization
- `Err(CredentialsError::PermissionDenied)` - User lacks `can_edit_credentials` permission
- `Err(CredentialsError::Encryption)` - Encryption failed
- `Err(CredentialsError::Database)` - Database operation failed

**Example:**
```rust
use flextide_core::credentials::update_credential;
use serde_json::json;

let updated_data = json!({
    "api_key": "sk-new-key-here",
    "base_url": "https://api.example.com"
});

update_credential(
    &pool,
    &manager,
    "credential-uuid-here",
    "org-uuid-here",
    "user-uuid-here",
    Some("Updated API Key"), // Optional: update name
    &updated_data
).await?;
```

#### `delete_credential(pool, credential_uuid, organization_uuid, user_uuid) -> Result<(), CredentialsError>`

Deletes a credential from the database.

**Arguments:**
- `pool: &DatabasePool` - Database connection pool
- `credential_uuid: &str` - UUID of the credential to delete
- `organization_uuid: &str` - UUID of the organization
- `user_uuid: &str` - UUID of the user deleting the credential

**Returns:**
- `Ok(())` - Success
- `Err(CredentialsError::CredentialNotFound)` - Credential doesn't exist
- `Err(CredentialsError::UserNotInOrganization)` - User doesn't belong to organization
- `Err(CredentialsError::PermissionDenied)` - User lacks `can_delete_credentials` permission
- `Err(CredentialsError::Database)` - Database operation failed

**Example:**
```rust
use flextide_core::credentials::delete_credential;

delete_credential(
    &pool,
    "credential-uuid-here",
    "org-uuid-here",
    "user-uuid-here"
).await?;
```

## Error Handling

All operations return `Result<T, CredentialsError>`. The error type includes:

- `Database(DatabaseError)` - Database connection or query errors
- `Sql(sqlx::Error)` - SQL execution errors
- `Encryption(String)` - Encryption operation failed
- `Decryption(String)` - Decryption operation failed
- `MasterKeyNotFound` - `CREDENTIALS_MASTER_KEY` environment variable not set
- `InvalidMasterKeyFormat` - Master key is not 64 hex characters
- `CredentialNotFound(String)` - Credential with given UUID not found
- `UserNotInOrganization` - User doesn't belong to the organization
- `PermissionDenied` - User lacks required permission
- `Serialization(serde_json::Error)` - JSON serialization/deserialization error
- `InvalidDataFormat` - Invalid credential data format

**Example:**
```rust
use flextide_core::credentials::{CredentialsError, get_credential};

match get_credential(&pool, &manager, uuid, org_uuid, user_uuid).await {
    Ok(credential) => {
        println!("Credential: {}", credential.name);
    }
    Err(CredentialsError::CredentialNotFound(uuid)) => {
        eprintln!("Credential {} not found", uuid);
    }
    Err(CredentialsError::PermissionDenied) => {
        eprintln!("Permission denied");
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

## Complete Example

```rust
use flextide_core::credentials::{
    CredentialsManager,
    create_credential,
    get_credential,
    list_credentials,
    update_credential,
    delete_credential,
};
use flextide_core::database::DatabasePool;
use serde_json::json;

async fn example_usage(pool: &DatabasePool) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize credentials manager
    let manager = CredentialsManager::new()?;

    let org_uuid = "organization-uuid";
    let user_uuid = "user-uuid";

    // Create a credential
    let credential_data = json!({
        "api_key": "sk-1234567890abcdef",
        "base_url": "https://api.example.com"
    });

    let credential_uuid = create_credential(
        &pool,
        &manager,
        org_uuid,
        user_uuid,
        "Production API Key",
        "openai",
        &credential_data
    ).await?;

    println!("Created credential: {}", credential_uuid);

    // List all credentials
    let credentials = list_credentials(&pool, org_uuid, user_uuid).await?;
    println!("Found {} credentials", credentials.len());

    // Get a specific credential
    let credential = get_credential(
        &pool,
        &manager,
        &credential_uuid,
        org_uuid,
        user_uuid
    ).await?;

    println!("Credential name: {}", credential.name);
    println!("Credential data: {}", credential.data);

    // Update credential
    let updated_data = json!({
        "api_key": "sk-new-key-here",
        "base_url": "https://api.example.com"
    });

    update_credential(
        &pool,
        &manager,
        &credential_uuid,
        org_uuid,
        user_uuid,
        Some("Updated API Key"),
        &updated_data
    ).await?;

    // Delete credential
    delete_credential(
        &pool,
        &credential_uuid,
        org_uuid,
        user_uuid
    ).await?;

    Ok(())
}
```

## Security Best Practices

1. **Master Key Management**
   - Never commit the master key to version control
   - Use different keys for different environments (dev/staging/prod)
   - Rotate the master key periodically
   - Store production keys in secure secret management services

2. **Access Control**
   - Always verify organization membership before operations
   - Require explicit permissions for all credential operations
   - Log all credential access for audit purposes

3. **Data Handling**
   - Never log credential data (even encrypted)
   - Clear sensitive data from memory when possible
   - Use HTTPS for all network communication

4. **Key Rotation**
   - Plan for key rotation (re-encrypt all credentials with new key)
   - Keep old key temporarily during rotation
   - Test rotation procedure in staging environment

## Database Schema

The credentials table structure:

```sql
CREATE TABLE credentials (
    uuid CHAR(36) NOT NULL PRIMARY KEY,
    organization_uuid CHAR(36) NOT NULL,
    name VARCHAR(255) NOT NULL,
    type VARCHAR(255) NOT NULL,
    encrypted_data BLOB NOT NULL,  -- Nonce (12 bytes) + encrypted JSON
    creator_user_uuid CHAR(36) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NULL,
    
    FOREIGN KEY (organization_uuid) REFERENCES organizations(uuid) ON DELETE CASCADE,
    FOREIGN KEY (creator_user_uuid) REFERENCES users(uuid) ON DELETE RESTRICT
);
```

## See Also

- [Credential Storage Research](../../../../docs/research/Credential_Storage.md) - Comprehensive security research
- [OWASP Cryptographic Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html)
- [AES-GCM Documentation](https://github.com/RustCrypto/AEADs/tree/master/aes-gcm)

