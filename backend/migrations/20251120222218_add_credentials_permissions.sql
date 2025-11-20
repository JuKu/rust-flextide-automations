-- Add Credentials permission group and permissions
-- Supports MySQL, PostgreSQL, and SQLite

-- ============================================================================
-- PERMISSION GROUP: CREDENTIALS
-- ============================================================================

INSERT INTO permission_groups (name, title, description, visible, sort_order)
SELECT new_groups.name, new_groups.title, new_groups.description, new_groups.visible, new_groups.sort_order
FROM (
    VALUES
        ('credentials', 'Credentials', 'Permissions for managing credentials (API keys, tokens, etc.)', 1, 6)
) AS new_groups(name, title, description, visible, sort_order)
WHERE NOT EXISTS (SELECT 1 FROM permission_groups WHERE permission_groups.name = new_groups.name);

-- ============================================================================
-- PERMISSIONS: CREDENTIALS
-- ============================================================================

INSERT INTO permissions (name, title, description, visible, sort_order, permission_group_name)
SELECT new_permissions.name, new_permissions.title, new_permissions.description, new_permissions.visible, new_permissions.sort_order, new_permissions.permission_group_name
FROM (
    VALUES
        ('can_see_all_credentials', 'Can see all credentials', 'The user is able to see all credentials of the organization (without their values)', 1, 1, 'credentials'),
        ('can_edit_credentials', 'Can edit credentials', 'The user is able to edit existing credentials', 1, 2, 'credentials'),
        ('can_delete_credentials', 'Can delete credentials', 'The user is able to delete credentials from the organization', 1, 3, 'credentials')
) AS new_permissions(name, title, description, visible, sort_order, permission_group_name)
WHERE NOT EXISTS (SELECT 1 FROM permissions WHERE permissions.name = new_permissions.name);

