-- Create Docs module tables: areas and area_members
-- Supports both MySQL and PostgreSQL
-- Note: updated_at is managed by application code, not database triggers

-- ============================================================================
-- PERMISSION GROUP
-- ============================================================================

-- Insert docs module permission group if it doesn't exist
INSERT INTO permission_groups (id, name, title, description, visible, sort_order)
SELECT new_groups.id, new_groups.name, new_groups.title, new_groups.description, new_groups.visible, new_groups.sort_order
FROM (
    VALUES
        ('00000000-0000-0000-0000-000000000006', 'module_docs', 'Docs', 'Permissions for the Docs Module', 1, 5)
) AS new_groups(id, name, title, description, visible, sort_order)
WHERE NOT EXISTS (SELECT 1 FROM permission_groups WHERE permission_groups.name = new_groups.name);

-- ============================================================================
-- PERMISSIONS
-- ============================================================================

-- Insert docs module permissions
INSERT INTO permissions (id, name, title, description, visible, sort_order, permission_group_name)
SELECT new_permissions.id, new_permissions.name, new_permissions.title, new_permissions.description, new_permissions.visible, new_permissions.sort_order, new_permissions.permission_group_name
FROM (
    VALUES
        ('10000000-0000-0000-0000-000000000100', 'module_docs_can_create_areas', 'Can create areas', 'The user is able to create new documentation areas in the organization', 1, 1, 'module_docs'),
        ('10000000-0000-0000-0000-000000000101', 'module_docs_can_edit_all_areas', 'Can edit all areas', 'The user is able to edit all documentation areas in the organization', 1, 2, 'module_docs'),
        ('10000000-0000-0000-0000-000000000102', 'module_docs_can_edit_own_areas', 'Can edit own areas', 'The user is able to edit documentation areas they created', 1, 3, 'module_docs'),
        ('10000000-0000-0000-0000-000000000103', 'module_docs_can_archive_areas', 'Can archive areas', 'The user is able to archive documentation areas in the organization', 1, 4, 'module_docs'),
        ('10000000-0000-0000-0000-000000000104', 'module_docs_can_archive_own_areas', 'Can archive own areas', 'The user is able to archive documentation areas they created', 1, 5, 'module_docs'),
        ('10000000-0000-0000-0000-000000000105', 'module_docs_can_delete_areas', 'Can delete areas', 'The user is able to delete documentation areas in the organization', 1, 6, 'module_docs'),
        ('10000000-0000-0000-0000-000000000106', 'module_docs_can_delete_own_areas', 'Can delete own areas', 'The user is able to delete documentation areas they created', 1, 7, 'module_docs')
) AS new_permissions(id, name, title, description, visible, sort_order, permission_group_name)
WHERE NOT EXISTS (SELECT 1 FROM permissions WHERE permissions.name = new_permissions.name);

-- ============================================================================
-- MODULE_DOCS_AREAS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS module_docs_areas (
    uuid CHAR(36) NOT NULL PRIMARY KEY,
    organization_uuid CHAR(36) NOT NULL,
    short_name VARCHAR(255) NOT NULL,
    description TEXT,
    icon_name VARCHAR(255),
    public INTEGER NOT NULL DEFAULT 0,
    visible INTEGER NOT NULL DEFAULT 1,
    deletable INTEGER NOT NULL DEFAULT 1,
    creator_uuid CHAR(36) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    archived INTEGER NOT NULL DEFAULT 0,
    activated INTEGER NOT NULL DEFAULT 1,
    FOREIGN KEY (organization_uuid) REFERENCES organizations(uuid) ON DELETE CASCADE,
    FOREIGN KEY (creator_uuid) REFERENCES users(uuid) ON DELETE RESTRICT
);

-- ============================================================================
-- MODULE_DOCS_AREA_MEMBERS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS module_docs_area_members (
    area_uuid CHAR(36) NOT NULL,
    user_uuid CHAR(36) NOT NULL,
    role VARCHAR(20) NOT NULL DEFAULT 'guest' CHECK (role IN ('owner', 'admin', 'member', 'guest')),
    can_view INTEGER NOT NULL DEFAULT 0,
    can_add_pages INTEGER NOT NULL DEFAULT 0,
    can_edit_pages INTEGER NOT NULL DEFAULT 0,
    can_edit_own_pages INTEGER NOT NULL DEFAULT 0,
    can_archive_pages INTEGER NOT NULL DEFAULT 0,
    can_archive_own_pages INTEGER NOT NULL DEFAULT 0,
    can_delete_pages INTEGER NOT NULL DEFAULT 0,
    can_delete_own_pages INTEGER NOT NULL DEFAULT 0,
    can_export_pages INTEGER NOT NULL DEFAULT 0,
    admin INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (area_uuid, user_uuid),
    FOREIGN KEY (area_uuid) REFERENCES module_docs_areas(uuid) ON DELETE CASCADE,
    FOREIGN KEY (user_uuid) REFERENCES users(uuid) ON DELETE CASCADE
);

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Indexes on module_docs_areas for efficient queries
CREATE INDEX IF NOT EXISTS idx_module_docs_areas_org ON module_docs_areas(organization_uuid);
CREATE INDEX IF NOT EXISTS idx_module_docs_areas_creator ON module_docs_areas(creator_uuid);
CREATE INDEX IF NOT EXISTS idx_module_docs_areas_public ON module_docs_areas(public);
CREATE INDEX IF NOT EXISTS idx_module_docs_areas_visible ON module_docs_areas(visible);
CREATE INDEX IF NOT EXISTS idx_module_docs_areas_created_at ON module_docs_areas(created_at);

-- Indexes on module_docs_area_members for efficient queries
CREATE INDEX IF NOT EXISTS idx_module_docs_area_members_area ON module_docs_area_members(area_uuid);
CREATE INDEX IF NOT EXISTS idx_module_docs_area_members_user ON module_docs_area_members(user_uuid);
CREATE INDEX IF NOT EXISTS idx_module_docs_area_members_role ON module_docs_area_members(role);
