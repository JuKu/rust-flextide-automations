-- Create permission_groups, permissions, and user_permissions tables
-- Supports both MySQL and PostgreSQL
--
-- This migration creates:
-- 1. permission_groups: Groups that organize permissions into logical categories (e.g., "CRM", "User Management")
-- 2. permissions: All available permissions a user can get
-- 3. user_permissions: Maps which permissions users have for specific organizations

-- ============================================================================
-- PERMISSION_GROUPS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS permission_groups (
    id CHAR(36) NOT NULL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    visible INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0
);

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Create index on sort_order field for efficient sorting
CREATE INDEX IF NOT EXISTS idx_permission_groups_sort_order ON permission_groups(sort_order);

-- ============================================================================
-- INITIAL DATA
-- ============================================================================

-- Insert initial permission groups
INSERT INTO permission_groups (id, name, title, description, visible, sort_order)
SELECT new_groups.id, new_groups.name, new_groups.title, new_groups.description, new_groups.visible, new_groups.sort_order
FROM (
    VALUES
        ('00000000-0000-0000-0000-000000000001', 'workflows', 'Workflows', 'Permission to create, edit, delete and execute workflows', 1, 1),
        ('00000000-0000-0000-0000-000000000002', 'ai_coworkers', 'AI Coworkers', 'Permissions to create, edit, delete and execute AI Coworkers', 1, 2),
        ('00000000-0000-0000-0000-000000000003', 'users', 'Users', 'Manage the members of this organization', 1, 3),
        ('00000000-0000-0000-0000-000000000004', 'module_crm', 'CRM', 'Permissions for the CRM Module', 1, 4),
        ('00000000-0000-0000-0000-000000000005', 'super_admin', 'Super Admin', 'Super administrator permissions that grant access to everything in an organization', 1, 0)
) AS new_groups(id, name, title, description, visible, sort_order)
WHERE NOT EXISTS (SELECT 1 FROM permission_groups WHERE permission_groups.name = new_groups.name);

-- ============================================================================
-- PERMISSIONS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS permissions (
    id CHAR(36) NOT NULL PRIMARY KEY,
    permission_group_name VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL UNIQUE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    visible INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (permission_group_name) REFERENCES permission_groups(name) ON DELETE RESTRICT
);

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Create index on sort_order field for efficient sorting
CREATE INDEX IF NOT EXISTS idx_permissions_sort_order ON permissions(sort_order);

-- Index on name for fast lookups
CREATE INDEX IF NOT EXISTS idx_permissions_name ON permissions(name);

-- Index on permission_group_name for efficient group lookups
CREATE INDEX IF NOT EXISTS idx_permissions_group_name ON permissions(permission_group_name);

-- ============================================================================
-- INITIAL PERMISSIONS DATA
-- ============================================================================

-- Insert CRM module permissions
-- All permissions belong to the 'module_crm' permission group
INSERT INTO permissions (id, name, title, description, visible, sort_order, permission_group_name)
SELECT new_permissions.id, new_permissions.name, new_permissions.title, new_permissions.description, new_permissions.visible, new_permissions.sort_order, new_permissions.permission_group_name
FROM (
    VALUES
        ('10000000-0000-0000-0000-000000000001', 'module_crm_can_create_customers', 'Can create customers', 'The user is able to create new customers in the CRM system', 1, 1, 'module_crm'),
        ('10000000-0000-0000-0000-000000000002', 'module_crm_can_edit_customers', 'Can edit customers', 'The user is able to edit customer details', 1, 2, 'module_crm'),
        ('10000000-0000-0000-0000-000000000003', 'module_crm_can_delete_customers', 'Can delete customers', 'The user is able to delete customers from the CRM system', 1, 3, 'module_crm'),
        ('10000000-0000-0000-0000-000000000004', 'module_crm_can_see_customer', 'Can see customer', 'The user is able to see the customer details page of a customer', 1, 4, 'module_crm'),
        ('10000000-0000-0000-0000-000000000005', 'module_crm_can_see_all_customers', 'Can see all customers', 'The user is able to see all customers of the organization with pagination', 1, 5, 'module_crm'),
        ('10000000-0000-0000-0000-000000000006', 'module_crm_search_customers', 'Can search customers', 'The user is able to search for customers in the CRM system', 1, 6, 'module_crm'),
        ('10000000-0000-0000-0000-000000000007', 'module_crm_can_add_customer_notes', 'Can add customer notes', 'The user is able to add notes to customers', 1, 7, 'module_crm'),
        ('10000000-0000-0000-0000-000000000008', 'module_crm_edit_customer_notes', 'Can edit customer notes', 'The user is able to edit notes attached to customers', 1, 8, 'module_crm'),
        ('10000000-0000-0000-0000-000000000009', 'module_crm_can_delete_customer_notes', 'Can delete customer notes', 'The user is able to delete notes from customers', 1, 9, 'module_crm'),
        ('10000000-0000-0000-0000-000000000010', 'module_crm_can_add_customer_addresses', 'Can add customer addresses', 'The user is able to add addresses to customers', 1, 10, 'module_crm'),
        ('10000000-0000-0000-0000-000000000011', 'module_crm_can_delete_customer_addresses', 'Can delete customer addresses', 'The user is able to delete addresses from customers', 1, 11, 'module_crm'),
        ('20000000-0000-0000-0000-000000000001', 'super_admin', 'Super Admin', 'Grants the user access to everything in the organization', 1, 1, 'super_admin')
) AS new_permissions(id, name, title, description, visible, sort_order, permission_group_name)
WHERE NOT EXISTS (SELECT 1 FROM permissions WHERE permissions.name = new_permissions.name);

-- ============================================================================
-- USER_PERMISSIONS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS user_permissions (
    user_id CHAR(36) NOT NULL,
    organization_uuid CHAR(36) NOT NULL,
    permission_name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, organization_uuid, permission_name),
    FOREIGN KEY (user_id) REFERENCES users(uuid) ON DELETE CASCADE,
    FOREIGN KEY (organization_uuid) REFERENCES organizations(uuid) ON DELETE CASCADE,
    FOREIGN KEY (permission_name) REFERENCES permissions(name) ON DELETE CASCADE
);

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Index on user_id for efficient user permission lookups
CREATE INDEX IF NOT EXISTS idx_user_permissions_user ON user_permissions(user_id);

-- Index on organization_uuid for efficient organization permission lookups
CREATE INDEX IF NOT EXISTS idx_user_permissions_org ON user_permissions(organization_uuid);

-- Index on permission_name for efficient permission lookups
CREATE INDEX IF NOT EXISTS idx_user_permissions_permission ON user_permissions(permission_name);

-- Composite index for efficient lookups of user permissions in an organization
CREATE INDEX IF NOT EXISTS idx_user_permissions_user_org ON user_permissions(user_id, organization_uuid);
