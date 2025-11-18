-- Create credentials table
-- Supports both MySQL and PostgreSQL
-- Note: updated_at is managed by application code, not database triggers
--
-- This table stores encrypted credentials (API keys, tokens, etc.) for external services.
-- Credentials are encrypted using AES-256-GCM before being stored in the database.
-- The encrypted_data column stores the nonce (12 bytes) + encrypted JSON data.

-- ============================================================================
-- CREDENTIALS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS credentials (
    uuid CHAR(36) NOT NULL PRIMARY KEY,
    organization_uuid CHAR(36) NOT NULL,
    name VARCHAR(255) NOT NULL,
    type VARCHAR(255) NOT NULL,
    encrypted_data BLOB NOT NULL,  -- Nonce (12 bytes) + encrypted JSON credential data
    salt VARCHAR(255) NULL,  -- Optional salt for additional security layer
    encryption_key_version INTEGER NOT NULL DEFAULT 1,  -- Version of encryption key used (for key rotation)
    creator_user_uuid CHAR(36) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NULL,
    
    FOREIGN KEY (organization_uuid) REFERENCES organizations(uuid) ON DELETE CASCADE,
    FOREIGN KEY (creator_user_uuid) REFERENCES users(uuid) ON DELETE RESTRICT
);

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Index on organization_uuid for efficient organization-scoped queries
CREATE INDEX IF NOT EXISTS idx_credentials_organization_uuid 
    ON credentials(organization_uuid);

-- Index on type for filtering credentials by type (e.g., "chroma", "openai")
CREATE INDEX IF NOT EXISTS idx_credentials_type 
    ON credentials(type);

-- Index on name for searching credentials by name
CREATE INDEX IF NOT EXISTS idx_credentials_name 
    ON credentials(name);

-- Composite index for efficient lookups of credentials by organization and type
CREATE INDEX IF NOT EXISTS idx_credentials_organization_type 
    ON credentials(organization_uuid, type);

-- ============================================================================
-- NOTES
-- ============================================================================
-- 
-- This table stores encrypted credentials for external services that can be
-- used by nodes in workflows. Credentials are encrypted at the application
-- level using AES-256-GCM before being stored in the database.
--
-- Fields:
-- - uuid: Unique identifier (UUID) for the credential
-- - organization_uuid: UUID of the organization that owns this credential
-- - name: Human-readable name for the credential (e.g., "Production Chroma API Key")
-- - type: Type of credential (e.g., "chroma", "openai", "github", "jira")
-- - encrypted_data: Binary blob containing nonce (12 bytes) + encrypted JSON data
--   The JSON data contains the actual credential information (API keys, tokens, etc.)
-- - salt: Optional salt value for additional security layer (can be used for key derivation)
-- - encryption_key_version: Version of the encryption key used (default: 1)
--   Used for key rotation - allows tracking which master key was used to encrypt each credential
-- - creator_user_uuid: UUID of the user who created this credential
-- - created_at: Timestamp when the credential was created
-- - updated_at: Timestamp when the credential was last updated (NULL if never updated)
--
-- Security:
-- - Credentials are encrypted using AES-256-GCM with a master key stored in
--   the CREDENTIALS_MASTER_KEY environment variable
-- - The master key is never stored in the database or code
-- - Each encryption uses a unique nonce, ensuring identical data produces different ciphertext
-- - Access control is enforced at the application level (permissions required)
--
-- Example credential data (before encryption):
-- {
--   "api_key": "sk-1234567890abcdef",
--   "base_url": "https://api.example.com",
--   "organization_id": "org-123"
-- }
--
-- After encryption, the encrypted_data column contains:
-- [12-byte nonce][encrypted JSON bytes]
