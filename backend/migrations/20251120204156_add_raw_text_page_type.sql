-- Add raw_text to page_type CHECK constraint in module_docs_pages table
-- Supports both MySQL and PostgreSQL

-- Add the new CHECK constraint with raw_text included
ALTER TABLE module_docs_pages
ADD CONSTRAINT module_docs_pages_page_type_check
CHECK (page_type IN ('markdown_page', 'json_document', 'database', 'sheet', 'raw_text'));
