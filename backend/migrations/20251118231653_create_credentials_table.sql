-- Create credentials and credential_types tables
-- Supports MySQL, PostgreSQL, and SQLite
-- Note: updated_at is managed by application code, not database triggers
--
-- This migration creates:
-- 1. credential_types table (must be created first)
-- 2. credentials table (references credential_types via foreign key)
-- 3. Initial credential types data

-- ============================================================================
-- CREDENTIAL TYPES TABLE (must be created first)
-- ============================================================================

CREATE TABLE IF NOT EXISTS credential_types (
    credential_type VARCHAR(255) NOT NULL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description VARCHAR(600) NOT NULL,
    icon_path VARCHAR(600) NULL,  -- Path to icon file (e.g., "/icons/chroma.svg")
    module_name VARCHAR(255) NULL,  -- Module that provides this credential type (e.g., "docs")
    plugin_name VARCHAR(255) NULL,  -- Plugin that provides this credential type
    integration_name VARCHAR(255) NULL,  -- Integration that provides this credential type (e.g., "chroma", "openai")
    visible INTEGER NOT NULL DEFAULT 1  -- 1 = visible, 0 = hidden
);

-- ============================================================================
-- CREDENTIAL TYPES INDEXES
-- ============================================================================

-- Index on visible for filtering visible credential types
CREATE INDEX IF NOT EXISTS idx_credential_types_visible 
    ON credential_types(visible);

-- Index on module_name for filtering by module
CREATE INDEX IF NOT EXISTS idx_credential_types_module_name 
    ON credential_types(module_name);

-- Index on plugin_name for filtering by plugin
CREATE INDEX IF NOT EXISTS idx_credential_types_plugin_name 
    ON credential_types(plugin_name);

-- Index on integration_name for filtering by integration
CREATE INDEX IF NOT EXISTS idx_credential_types_integration_name 
    ON credential_types(integration_name);

-- ============================================================================
-- INITIAL CREDENTIAL TYPES DATA (cross-database compatible)
-- ============================================================================

-- Insert default credential types using INSERT ... SELECT ... FROM (VALUES ...) pattern
-- This works for MySQL, PostgreSQL, and SQLite without modification
-- Uses VALUES syntax combined with WHERE NOT EXISTS for conflict handling

INSERT INTO credential_types (credential_type, title, description, icon_path, module_name, plugin_name, integration_name, visible)
SELECT new_types.credential_type, new_types.title, new_types.description, new_types.icon_path, new_types.module_name, new_types.plugin_name, new_types.integration_name, new_types.visible
FROM (
    VALUES
        ('openai_credential', 'OpenAI API Key', 'Credentials to access OpenAI API', NULL, NULL, NULL, 'openai', 1),
        ('jira_credential', 'JIRA API Key', 'Credentials to access JIRA REST API', NULL, NULL, NULL, 'jira', 1),
        ('github_credential', 'GitHub API Key', 'Credentials to access GitHub REST API', NULL, NULL, NULL, 'github', 1),
        ('chroma_credential', 'Chroma Vector Database API Key', 'Credentials to access Chroma Vector Database API', NULL, NULL, NULL, 'chroma', 1)
) AS new_types(credential_type, title, description, icon_path, module_name, plugin_name, integration_name, visible)
WHERE NOT EXISTS (SELECT 1 FROM credential_types WHERE credential_types.credential_type = new_types.credential_type);

-- ============================================================================
-- CREDENTIALS TABLE (created after credential_types due to foreign key)
-- ============================================================================

CREATE TABLE IF NOT EXISTS credentials (
    uuid CHAR(36) NOT NULL PRIMARY KEY,
    organization_uuid CHAR(36) NOT NULL,
    name VARCHAR(255) NOT NULL,
    credential_type VARCHAR(255) NOT NULL,  -- Renamed from "type" to avoid reserved word conflicts
    encrypted_data BLOB NOT NULL,  -- Nonce (12 bytes) + encrypted JSON credential data
    salt VARCHAR(255) NULL,  -- Optional salt for additional security layer
    encryption_key_version INTEGER NOT NULL DEFAULT 1,  -- Version of encryption key used (for key rotation)
    creator_user_uuid CHAR(36) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NULL,
    
    FOREIGN KEY (organization_uuid) REFERENCES organizations(uuid) ON DELETE CASCADE,
    FOREIGN KEY (creator_user_uuid) REFERENCES users(uuid) ON DELETE RESTRICT,
    FOREIGN KEY (credential_type) REFERENCES credential_types(credential_type) ON DELETE RESTRICT
);

-- ============================================================================
-- CREDENTIALS INDEXES
-- ============================================================================

-- Index on organization_uuid for efficient organization-scoped queries
CREATE INDEX IF NOT EXISTS idx_credentials_organization_uuid 
    ON credentials(organization_uuid);

-- Index on credential_type for filtering credentials by type (e.g., "chroma_credential", "openai_credential")
CREATE INDEX IF NOT EXISTS idx_credentials_credential_type 
    ON credentials(credential_type);

-- Index on name for searching credentials by name
CREATE INDEX IF NOT EXISTS idx_credentials_name 
    ON credentials(name);

-- Composite index for efficient lookups of credentials by organization and credential type
CREATE INDEX IF NOT EXISTS idx_credentials_organization_credential_type 
    ON credentials(organization_uuid, credential_type);

-- ============================================================================
-- NOTES
-- ============================================================================
-- 
-- CREDENTIALS TABLE:
-- This table stores encrypted credentials for external services that can be
-- used by nodes in workflows. Credentials are encrypted at the application
-- level using AES-256-GCM before being stored in the database.
--
-- Fields:
-- - uuid: Unique identifier (UUID) for the credential
-- - organization_uuid: UUID of the organization that owns this credential
-- - name: Human-readable name for the credential (e.g., "Production Chroma API Key")
-- - credential_type: Type of credential (e.g., "chroma_credential", "openai_credential", "github_credential", "jira_credential")
--   References credential_types.credential_type via foreign key
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
--
-- CREDENTIAL TYPES TABLE:
-- This table stores metadata about different credential types that can be
-- used in the system. It provides information about available credential
-- types, their display names, descriptions, icons, and which module/plugin
-- or integration provides them.
--
-- Fields:
-- - credential_type: Unique identifier for the credential type (e.g., "chroma_credential", "openai_credential", "github_credential")
--   This matches the "credential_type" field in the credentials table (enforced via foreign key)
-- - title: Human-readable title for the credential type (e.g., "Chroma Vector Database")
-- - description: Detailed description of what this credential type is used for
-- - icon_path: Optional path to an icon file for displaying this credential type in the UI
-- - module_name: Optional name of the module that provides this credential type
-- - plugin_name: Optional name of the plugin that provides this credential type
-- - integration_name: Optional name of the integration that provides this credential type
-- - visible: Whether this credential type should be visible in the UI (1 = visible, 0 = hidden)
--
-- Usage:
-- - This table is used to populate credential type selection dropdowns in the UI
-- - Only visible credential types are shown to users
-- - The credential_type value must match the "credential_type" field in the credentials table (enforced via foreign key)
-- - Modules, plugins, or integrations can register their credential types here
-- - Foreign key constraint ensures only valid credential types can be used in credentials table
--
-- Initial Data:
-- The migration includes INSERT statements to populate default credential types:
-- - openai_credential: OpenAI API Key
-- - jira_credential: JIRA API Key
-- - github_credential: GitHub API Key
-- - chroma_credential: Chroma Vector Database API Key
