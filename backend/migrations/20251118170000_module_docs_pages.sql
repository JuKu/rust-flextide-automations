-- Create Docs module tables: folders, pages and page_versions
-- Supports both MySQL and PostgreSQL
-- Note: updated_at is managed by application code, not database triggers

-- ============================================================================
-- MODULE_DOCS_FOLDERS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS module_docs_folders (
    uuid CHAR(36) NOT NULL PRIMARY KEY,
    organization_uuid CHAR(36) NOT NULL,
    area_uuid CHAR(36) NOT NULL,
    name VARCHAR(255) NOT NULL,
    icon_name VARCHAR(50) NULL,
    folder_color VARCHAR(20) NULL,
    parent_folder_uuid CHAR(36),
    sort_order INTEGER NOT NULL DEFAULT 0,
    visible INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    activated INTEGER NOT NULL DEFAULT 1,
    FOREIGN KEY (organization_uuid) REFERENCES organizations(uuid) ON DELETE CASCADE,
    FOREIGN KEY (area_uuid) REFERENCES module_docs_areas(uuid) ON DELETE CASCADE,
    FOREIGN KEY (parent_folder_uuid) REFERENCES module_docs_folders(uuid) ON DELETE SET NULL
);

-- ============================================================================
-- MODULE_DOCS_PAGES TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS module_docs_pages (
    uuid CHAR(36) NOT NULL PRIMARY KEY,
    organization_uuid CHAR(36) NOT NULL,
    area_uuid CHAR(36) NOT NULL,
    folder_uuid CHAR(36),
    title VARCHAR(255) NOT NULL,
    short_summary TEXT,
    parent_page_uuid CHAR(36),
    current_version_uuid CHAR(36),
    page_type VARCHAR(50) NOT NULL DEFAULT 'markdown_page' CHECK (page_type IN ('markdown_page', 'json_document', 'database', 'sheet')),
    last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (organization_uuid) REFERENCES organizations(uuid) ON DELETE CASCADE,
    FOREIGN KEY (area_uuid) REFERENCES module_docs_areas(uuid) ON DELETE CASCADE,
    FOREIGN KEY (folder_uuid) REFERENCES module_docs_folders(uuid) ON DELETE SET NULL,
    FOREIGN KEY (parent_page_uuid) REFERENCES module_docs_pages(uuid) ON DELETE SET NULL
);

-- ============================================================================
-- MODULE_DOCS_PAGE_VERSIONS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS module_docs_page_versions (
    uuid CHAR(36) NOT NULL PRIMARY KEY,
    page_uuid CHAR(36) NOT NULL,
    version_number INTEGER NOT NULL DEFAULT 1,
    content TEXT NOT NULL,
    last_updated TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (page_uuid) REFERENCES module_docs_pages(uuid) ON DELETE CASCADE,
    CONSTRAINT unique_page_version UNIQUE (page_uuid, version_number)
);

-- ============================================================================
-- ADD FOREIGN KEY FOR CURRENT_VERSION_UUID
-- ============================================================================

-- Add foreign key constraint for current_version_uuid after versions table is created
-- Note: This uses ALTER TABLE which works on both MySQL and PostgreSQL
ALTER TABLE module_docs_pages
ADD CONSTRAINT fk_module_docs_pages_current_version
FOREIGN KEY (current_version_uuid) REFERENCES module_docs_page_versions(uuid) ON DELETE SET NULL;

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Indexes on module_docs_folders for efficient queries
CREATE INDEX IF NOT EXISTS idx_module_docs_folders_org ON module_docs_folders(organization_uuid);
CREATE INDEX IF NOT EXISTS idx_module_docs_folders_area ON module_docs_folders(area_uuid);
CREATE INDEX IF NOT EXISTS idx_module_docs_folders_parent ON module_docs_folders(parent_folder_uuid);
CREATE INDEX IF NOT EXISTS idx_module_docs_folders_visible ON module_docs_folders(visible);
CREATE INDEX IF NOT EXISTS idx_module_docs_folders_activated ON module_docs_folders(activated);
CREATE INDEX IF NOT EXISTS idx_module_docs_folders_sort_order ON module_docs_folders(area_uuid, parent_folder_uuid, sort_order);
CREATE INDEX IF NOT EXISTS idx_module_docs_folders_created_at ON module_docs_folders(created_at);

-- Indexes on module_docs_pages for efficient queries
CREATE INDEX IF NOT EXISTS idx_module_docs_pages_org ON module_docs_pages(organization_uuid);
CREATE INDEX IF NOT EXISTS idx_module_docs_pages_area ON module_docs_pages(area_uuid);
CREATE INDEX IF NOT EXISTS idx_module_docs_pages_folder ON module_docs_pages(folder_uuid);
CREATE INDEX IF NOT EXISTS idx_module_docs_pages_parent ON module_docs_pages(parent_page_uuid);
CREATE INDEX IF NOT EXISTS idx_module_docs_pages_current_version ON module_docs_pages(current_version_uuid);
CREATE INDEX IF NOT EXISTS idx_module_docs_pages_type ON module_docs_pages(page_type);
CREATE INDEX IF NOT EXISTS idx_module_docs_pages_created_at ON module_docs_pages(created_at);
CREATE INDEX IF NOT EXISTS idx_module_docs_pages_last_updated ON module_docs_pages(last_updated);

-- Indexes on module_docs_page_versions for efficient queries
CREATE INDEX IF NOT EXISTS idx_module_docs_page_versions_page ON module_docs_page_versions(page_uuid);
CREATE INDEX IF NOT EXISTS idx_module_docs_page_versions_number ON module_docs_page_versions(page_uuid, version_number);
CREATE INDEX IF NOT EXISTS idx_module_docs_page_versions_created_at ON module_docs_page_versions(created_at);

