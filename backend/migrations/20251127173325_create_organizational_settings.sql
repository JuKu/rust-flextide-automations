-- Create organizational settings tables
-- Supports both MySQL and PostgreSQL
-- Note: updated_at is managed by application code, not database triggers
--
-- This migration creates three tables:
-- 1. organizational_settings_groups - Groups of settings (e.g., "Docs", "CRM")
-- 2. organizational_settings - Available settings definitions (independent of organization)
-- 3. organizational_settings_values - Actual values for each organization

-- ============================================================================
-- ORGANIZATIONAL_SETTINGS_GROUPS TABLE
-- ============================================================================
-- Groups settings into logical categories (e.g., "Docs", "CRM")
-- These groups are the same for all organizations

CREATE TABLE IF NOT EXISTS organizational_settings_groups (
    unique_name VARCHAR(255) NOT NULL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- ORGANIZATIONAL_SETTINGS TABLE
-- ============================================================================
-- Defines all available settings (independent of organization)
-- Each setting belongs to a settings group

CREATE TABLE IF NOT EXISTS organizational_settings (
    name VARCHAR(255) NOT NULL PRIMARY KEY,
    organizational_settings_group_name VARCHAR(255) NOT NULL,
    title VARCHAR(255) NOT NULL,
    description VARCHAR(255),
    type VARCHAR(50) NOT NULL CHECK (type IN ('dropdown', 'textfield', 'textarea', 'date', 'color', 'checkbox')),
    metadata JSON,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (organizational_settings_group_name) REFERENCES organizational_settings_groups(unique_name) ON DELETE CASCADE
);

-- ============================================================================
-- ORGANIZATIONAL_SETTINGS_VALUES TABLE
-- ============================================================================
-- Stores the actual values for each organization and setting
-- Each row represents one setting value for one organization

CREATE TABLE IF NOT EXISTS organizational_settings_values (
    organization_uuid CHAR(36) NOT NULL,
    setting_name VARCHAR(255) NOT NULL,
    value VARCHAR(600),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (setting_name, organization_uuid),
    FOREIGN KEY (organization_uuid) REFERENCES organizations(uuid) ON DELETE CASCADE,
    FOREIGN KEY (setting_name) REFERENCES organizational_settings(name) ON DELETE CASCADE
);

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Index on organizational_settings for efficient group lookups
CREATE INDEX IF NOT EXISTS idx_organizational_settings_group ON organizational_settings(organizational_settings_group_name);

-- Index on organizational_settings_values for efficient organization lookups
CREATE INDEX IF NOT EXISTS idx_organizational_settings_values_org ON organizational_settings_values(organization_uuid);

-- Index on organizational_settings_values for efficient setting lookups
CREATE INDEX IF NOT EXISTS idx_organizational_settings_values_setting ON organizational_settings_values(setting_name);

-- Composite index for efficient lookups by organization and setting
CREATE INDEX IF NOT EXISTS idx_organizational_settings_values_org_setting ON organizational_settings_values(organization_uuid, setting_name);

