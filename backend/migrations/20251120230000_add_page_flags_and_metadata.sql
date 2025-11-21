-- Add flags and metadata columns to module_docs_pages table
-- Supports both MySQL and PostgreSQL

ALTER TABLE module_docs_pages
ADD COLUMN auto_sync_to_vector_db INTEGER NOT NULL DEFAULT 0,
ADD COLUMN vcs_export_allowed INTEGER NOT NULL DEFAULT 0,
ADD COLUMN includes_private_data INTEGER NOT NULL DEFAULT 0,
ADD COLUMN metadata JSON;

-- Add indexes for the flag columns for efficient filtering
CREATE INDEX IF NOT EXISTS idx_module_docs_pages_auto_sync ON module_docs_pages(auto_sync_to_vector_db);
CREATE INDEX IF NOT EXISTS idx_module_docs_pages_vcs_export ON module_docs_pages(vcs_export_allowed);
CREATE INDEX IF NOT EXISTS idx_module_docs_pages_private_data ON module_docs_pages(includes_private_data);

