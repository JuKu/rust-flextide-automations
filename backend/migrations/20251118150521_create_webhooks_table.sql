-- Create event_webhooks table
-- Supports both MySQL and PostgreSQL
-- Note: updated_at is managed by application code, not database triggers
--
-- This table stores webhook configurations for event subscriptions.
-- Webhooks are organization-scoped and only receive events from their organization.

-- ============================================================================
-- EVENT_WEBHOOKS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS event_webhooks (
    id CHAR(36) NOT NULL PRIMARY KEY,
    
    -- Organization this webhook belongs to (required for filtering)
    organization_uuid CHAR(36) NOT NULL,
    
    -- Event name to subscribe to (e.g., "core_organization_created", "module_crm_customer_created")
    event_name VARCHAR(255) NOT NULL,
    
    -- Webhook endpoint URL
    url VARCHAR(2048) NOT NULL,
    
    -- Optional secret for HMAC signature verification
    -- If provided, webhook payloads will include X-Webhook-Signature header
    secret VARCHAR(255) NULL,
    
    -- Optional custom headers (JSON object)
    -- Example: {"Authorization": "Bearer token", "X-Custom-Header": "value"}
    headers JSON NULL,
    
    -- Whether the webhook is active
    active INTEGER NOT NULL DEFAULT 1, -- 1 = active, 0 = inactive (works in both databases)
    
    -- User who created this webhook
    created_by CHAR(36) NOT NULL,
    
    -- Timestamps
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign keys
    FOREIGN KEY (organization_uuid) REFERENCES organizations(uuid) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users(uuid) ON DELETE RESTRICT
);

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Index on organization_uuid for efficient organization-scoped lookups
CREATE INDEX IF NOT EXISTS idx_event_webhooks_organization_uuid 
    ON event_webhooks(organization_uuid);

-- Index on event_name for efficient event-based lookups
CREATE INDEX IF NOT EXISTS idx_event_webhooks_event_name 
    ON event_webhooks(event_name);

-- Index on active for filtering active webhooks
CREATE INDEX IF NOT EXISTS idx_event_webhooks_active 
    ON event_webhooks(active);

-- Composite index for efficient lookups of active webhooks for a specific event and organization
CREATE INDEX IF NOT EXISTS idx_event_webhooks_event_org_active 
    ON event_webhooks(event_name, organization_uuid, active);

-- Index on created_by for user-based queries
CREATE INDEX IF NOT EXISTS idx_event_webhooks_created_by 
    ON event_webhooks(created_by);

-- ============================================================================
-- NOTES
-- ============================================================================
-- 
-- This table stores webhook configurations that are loaded into memory at
-- application startup. Webhooks are organization-scoped, meaning:
-- 1. Users can only create webhooks for their own organization
-- 2. Webhooks only receive events from their organization
-- 3. Events are filtered by organization_uuid before sending to webhook endpoints
--
-- Fields:
-- - id: Unique identifier (UUID)
-- - organization_uuid: Organization this webhook belongs to (required)
-- - event_name: Name of the event to subscribe to (e.g., "module_crm_customer_created")
-- - url: Webhook endpoint URL (must be HTTPS in production)
-- - secret: Optional secret for HMAC signature verification
-- - headers: Optional JSON object with custom headers to include in webhook requests
-- - active: Whether the webhook is active (1 = active, 0 = inactive)
-- - created_by: UUID of the user who created this webhook
-- - created_at: Timestamp when the webhook was created
-- - updated_at: Timestamp when the webhook was last updated (managed by application)
--
-- Example webhook:
-- {
--   "id": "uuid-here",
--   "organization_uuid": "org-uuid-here",
--   "event_name": "module_crm_customer_created",
--   "url": "https://example.com/webhooks/customer-created",
--   "secret": "webhook-secret-key",
--   "headers": {"Authorization": "Bearer token"},
--   "active": 1,
--   "created_by": "user-uuid-here"
-- }

