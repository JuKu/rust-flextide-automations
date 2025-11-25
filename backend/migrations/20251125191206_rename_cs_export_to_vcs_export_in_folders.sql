-- Rename cs_export_allowed column to vcs_export_allowed in module_docs_folders table
-- Supports MySQL, PostgreSQL, and SQLite
-- 
-- IMPORTANT: This migration requires that migration 20251125182859_add_folder_flags_and_metadata.sql
-- has been run first. If you get an error that the column doesn't exist, run that migration first.

-- Rename the column
-- For MySQL < 8.0.2, use: ALTER TABLE module_docs_folders CHANGE COLUMN cs_export_allowed vcs_export_allowed INTEGER NOT NULL DEFAULT 0;
-- For MySQL 8.0.2+, PostgreSQL, and SQLite 3.25.0+, use RENAME COLUMN:
ALTER TABLE module_docs_folders
RENAME COLUMN cs_export_allowed TO vcs_export_allowed;

-- Create the new index with the correct name (works on all databases)
CREATE INDEX IF NOT EXISTS idx_module_docs_folders_vcs_export ON module_docs_folders(vcs_export_allowed);

-- Note: The old index idx_module_docs_folders_cs_export will become invalid after the column rename.
-- It can be manually cleaned up later if desired:
-- MySQL: DROP INDEX idx_module_docs_folders_cs_export ON module_docs_folders;
-- PostgreSQL: DROP INDEX IF EXISTS idx_module_docs_folders_cs_export;
-- SQLite: DROP INDEX IF EXISTS idx_module_docs_folders_cs_export;
