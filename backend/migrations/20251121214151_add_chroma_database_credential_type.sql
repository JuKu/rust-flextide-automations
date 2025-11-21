-- Add chroma_database credential type
-- This credential type is used for Chroma database connections
-- Supports MySQL, PostgreSQL, and SQLite
--
-- Uses the same cross-database compatible pattern as the initial credential_types migration

INSERT INTO credential_types (credential_type, title, description, icon_path, module_name, plugin_name, integration_name, visible)
SELECT new_types.credential_type, new_types.title, new_types.description, new_types.icon_path, new_types.module_name, new_types.plugin_name, new_types.integration_name, new_types.visible
FROM (
    VALUES
        ('chroma_database', 'Chroma Database Connection', 'Connection credentials for Chroma vector database', NULL, NULL, NULL, 'chroma', 1)
) AS new_types(credential_type, title, description, icon_path, module_name, plugin_name, integration_name, visible)
WHERE NOT EXISTS (SELECT 1 FROM credential_types WHERE credential_types.credential_type = new_types.credential_type);
