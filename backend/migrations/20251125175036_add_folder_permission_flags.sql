-- Add folder and property permission flags to module_docs_area_members table
-- Supports both MySQL and PostgreSQL

-- Add new permission columns
ALTER TABLE module_docs_area_members
ADD COLUMN can_add_folders INTEGER NOT NULL DEFAULT 0,
ADD COLUMN can_edit_folders INTEGER NOT NULL DEFAULT 0,
ADD COLUMN can_delete_folders INTEGER NOT NULL DEFAULT 0,
ADD COLUMN can_edit_page_properties INTEGER NOT NULL DEFAULT 0,
ADD COLUMN can_edit_folder_properties INTEGER NOT NULL DEFAULT 0;

-- Set default permissions for existing owners and admins
-- Owners and admins get all permissions by default
UPDATE module_docs_area_members
SET 
    can_add_folders = 1,
    can_edit_folders = 1,
    can_delete_folders = 1,
    can_edit_page_properties = 1,
    can_edit_folder_properties = 1
WHERE role IN ('owner', 'admin') OR admin = 1;
