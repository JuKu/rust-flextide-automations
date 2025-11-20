-- Add color_hex and topics columns to module_docs_areas table
-- Supports both MySQL and PostgreSQL
--
-- This migration adds:
-- 1. color_hex: Optional hex color code for the area (e.g., "#FF5733")
-- 2. topics: Optional comma-separated topics/labels for the area (e.g., "AI, Machine Learning, Documentation")

-- ============================================================================
-- ADD COLUMNS TO MODULE_DOCS_AREAS TABLE
-- ============================================================================

-- Add color_hex column (VARCHAR(7) to store hex color codes like #FF5733)
ALTER TABLE module_docs_areas
ADD COLUMN IF NOT EXISTS color_hex VARCHAR(7) NULL;

-- Add topics column (TEXT to store comma-separated topics)
ALTER TABLE module_docs_areas
ADD COLUMN IF NOT EXISTS topics TEXT NULL;

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Index on color_hex for filtering areas by color (optional, but useful for UI filtering)
CREATE INDEX IF NOT EXISTS idx_module_docs_areas_color_hex 
    ON module_docs_areas(color_hex);

-- Note: Full-text search on topics could be added later if needed
-- For now, we'll rely on application-level filtering

