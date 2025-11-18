-- Create event_subscriptions table
-- Supports both MySQL and PostgreSQL
-- Note: updated_at is managed by application code, not database triggers
--
-- This table stores event subscriptions that can be loaded into memory
-- at application startup for efficient event dispatching.

-- ============================================================================
-- EVENT SUBSCRIPTIONS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS event_subscriptions (
    id CHAR(36) NOT NULL PRIMARY KEY,
    event_name VARCHAR(255) NOT NULL,
    subscriber_type VARCHAR(50) NOT NULL,
    config JSON NOT NULL, -- JSON configuration (works in both MySQL and PostgreSQL)
    active INTEGER NOT NULL DEFAULT 1, -- 1 = active, 0 = inactive (works in both databases)
    organization_uuid CHAR(36) NULL,
    created_from VARCHAR(255) NOT NULL DEFAULT 'system',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Index on event_name for efficient event subscription lookups
CREATE INDEX IF NOT EXISTS idx_event_subscriptions_event_name 
    ON event_subscriptions(event_name);

-- Index on active for filtering active subscriptions
CREATE INDEX IF NOT EXISTS idx_event_subscriptions_active 
    ON event_subscriptions(active);

-- Index on organization_uuid for organization-scoped subscriptions
CREATE INDEX IF NOT EXISTS idx_event_subscriptions_organization_uuid 
    ON event_subscriptions(organization_uuid);

-- Index on created_from for filtering subscriptions by source
CREATE INDEX IF NOT EXISTS idx_event_subscriptions_created_from 
    ON event_subscriptions(created_from);

-- Composite index for efficient lookups of active subscriptions for a specific event
CREATE INDEX IF NOT EXISTS idx_event_subscriptions_event_active 
    ON event_subscriptions(event_name, active);

-- ============================================================================
-- NOTES
-- ============================================================================
-- 
-- This table stores event subscriptions that are loaded into memory at
-- application startup. The subscriptions are cached in a Map<EventName, List<Subscriber>>
-- structure for efficient event dispatching.
--
-- Fields:
-- - id: Unique identifier (UUID)
-- - event_name: Name of the event to subscribe to (e.g., "project.created")
-- - subscriber_type: Type of subscriber (e.g., "webhook", "kafka", "function")
-- - config: JSON configuration for the subscriber (e.g., webhook URL, Kafka topic)
-- - active: Whether the subscription is active (1 = active, 0 = inactive)
-- - organization_uuid: Optional organization UUID for organization-scoped subscriptions
-- - created_from: Source that created this subscription (e.g., "plugin_name", "system")
-- - created_at: Timestamp when the subscription was created
-- - updated_at: Timestamp when the subscription was last updated (managed by application)
--
-- Example subscription for a webhook:
-- {
--   "id": "uuid-here",
--   "event_name": "project.created",
--   "subscriber_type": "webhook",
--   "config": {"url": "https://example.com/webhook", "method": "POST"},
--   "active": 1,
--   "organization_uuid": null,
--   "created_from": "system"
-- }

