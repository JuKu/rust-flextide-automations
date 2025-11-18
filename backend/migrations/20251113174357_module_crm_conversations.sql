-- Create CRM module tables: conversation channels and customer conversations
-- Supports both MySQL and PostgreSQL
-- Note: updated_at is managed by application code, not database triggers

-- ============================================================================
-- MODULE_CRM_CONVERSATION_CHANNELS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS module_crm_conversation_channels (
    channel_uuid CHAR(36) NOT NULL PRIMARY KEY,
    organization_uuid CHAR(36) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description VARCHAR(600),
    icon_name VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (organization_uuid) REFERENCES organizations(uuid) ON DELETE CASCADE
);

-- ============================================================================
-- MODULE_CRM_CUSTOMER_CONVERSATIONS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS module_crm_customer_conversations (
    conversation_uuid CHAR(36) NOT NULL PRIMARY KEY,
    customer_uuid CHAR(36) NOT NULL,
    message TEXT NOT NULL,
    source VARCHAR(20) NOT NULL CHECK (source IN ('FROM_TEAM', 'FROM_CUSTOMER', 'INTERNAL_NOTE')),
    channel_uuid CHAR(36) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (customer_uuid) REFERENCES module_crm_customers(uuid) ON DELETE CASCADE,
    FOREIGN KEY (channel_uuid) REFERENCES module_crm_conversation_channels(channel_uuid) ON DELETE RESTRICT
);

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Indexes on module_crm_conversation_channels for efficient queries
CREATE INDEX IF NOT EXISTS idx_module_crm_conversation_channels_org ON module_crm_conversation_channels(organization_uuid);
CREATE INDEX IF NOT EXISTS idx_module_crm_conversation_channels_created_at ON module_crm_conversation_channels(created_at);

-- Indexes on module_crm_customer_conversations for efficient queries
CREATE INDEX IF NOT EXISTS idx_module_crm_customer_conversations_customer ON module_crm_customer_conversations(customer_uuid);
CREATE INDEX IF NOT EXISTS idx_module_crm_customer_conversations_channel ON module_crm_customer_conversations(channel_uuid);
CREATE INDEX IF NOT EXISTS idx_module_crm_customer_conversations_source ON module_crm_customer_conversations(source);
CREATE INDEX IF NOT EXISTS idx_module_crm_customer_conversations_created_at ON module_crm_customer_conversations(created_at);

