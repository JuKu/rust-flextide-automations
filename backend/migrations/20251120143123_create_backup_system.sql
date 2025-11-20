-- Create backup system tables and permissions
-- Supports both MySQL and PostgreSQL
-- Note: updated_at is managed by application code, not database triggers
--
-- This migration creates:
-- 1. backups table: Stores all backup records
-- 2. backup_jobs table: Stores scheduled backup jobs
-- 3. Backup permission group and permissions

-- ============================================================================
-- BACKUPS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS backups (
    uuid CHAR(36) NOT NULL PRIMARY KEY,
    filename VARCHAR(500) NOT NULL,
    full_path TEXT NOT NULL,
    creator_user_uuid CHAR(36) NOT NULL,
    target_location VARCHAR(100) NOT NULL DEFAULT 'local_filesystem',
    backup_status VARCHAR(50) NOT NULL DEFAULT 'COMPLETED' CHECK (backup_status IN ('COMPLETED', 'FAILED', 'IN_PROGRESS', 'CANCELLED')),
    backup_hash_checksum VARCHAR(128),
    is_encrypted INTEGER NOT NULL DEFAULT 0,
    encryption_algorithm VARCHAR(50),
    encryption_master_key_name VARCHAR(255),
    start_timestamp TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (creator_user_uuid) REFERENCES users(uuid) ON DELETE RESTRICT
);

-- ============================================================================
-- BACKUP_JOBS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS backup_jobs (
    uuid CHAR(36) NOT NULL PRIMARY KEY,
    job_type VARCHAR(50) NOT NULL,
    job_title VARCHAR(255) NOT NULL,
    json_data TEXT,
    last_execution_timestamp TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- INDEXES FOR BACKUPS TABLE
-- ============================================================================

-- Index on creator_user_uuid for efficient user lookups
CREATE INDEX IF NOT EXISTS idx_backups_creator ON backups(creator_user_uuid);

-- Index on backup_status for filtering by status
CREATE INDEX IF NOT EXISTS idx_backups_status ON backups(backup_status);

-- Index on target_location for filtering by location
CREATE INDEX IF NOT EXISTS idx_backups_target_location ON backups(target_location);

-- Index on created_at for sorting and filtering
CREATE INDEX IF NOT EXISTS idx_backups_created_at ON backups(created_at);

-- Index on start_timestamp for sorting
CREATE INDEX IF NOT EXISTS idx_backups_start_timestamp ON backups(start_timestamp);

-- ============================================================================
-- INDEXES FOR BACKUP_JOBS TABLE
-- ============================================================================

-- Index on job_type for filtering by type
CREATE INDEX IF NOT EXISTS idx_backup_jobs_type ON backup_jobs(job_type);

-- Index on last_execution_timestamp for sorting
CREATE INDEX IF NOT EXISTS idx_backup_jobs_last_execution ON backup_jobs(last_execution_timestamp);

-- Index on created_at for sorting
CREATE INDEX IF NOT EXISTS idx_backup_jobs_created_at ON backup_jobs(created_at);

-- ============================================================================
-- PERMISSION GROUP
-- ============================================================================

-- Insert backup permission group
-- MySQL-compatible version using INSERT IGNORE
INSERT IGNORE INTO permission_groups (id, name, title, description, visible, sort_order)
VALUES ('00000000-0000-0000-0000-000000000006', 'backup', 'Backup', 'Permissions for backup management', 1, 6);

-- ============================================================================
-- PERMISSIONS
-- ============================================================================

-- Insert backup permissions
-- MySQL-compatible version using INSERT IGNORE
INSERT IGNORE INTO permissions (id, name, title, description, visible, sort_order, permission_group_name)
VALUES
    ('30000000-0000-0000-0000-000000000001', 'can_create_backup', 'Can create backup', 'The user is able to create new backups', 1, 1, 'backup'),
    ('30000000-0000-0000-0000-000000000002', 'can_see_all_backups', 'Can see all backups', 'The user is able to see all backups with pagination', 1, 2, 'backup'),
    ('30000000-0000-0000-0000-000000000003', 'can_restore_backup', 'Can restore backup', 'The user is able to restore backups', 1, 3, 'backup'),
    ('30000000-0000-0000-0000-000000000004', 'can_download_backup', 'Can download backup', 'The user is able to download backup files', 1, 4, 'backup'),
    ('30000000-0000-0000-0000-000000000005', 'can_delete_backup', 'Can delete backup', 'The user is able to delete backups', 1, 5, 'backup');

