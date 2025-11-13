-- Create users, organizations, and organization_members tables
-- Supports both MySQL and PostgreSQL
--
-- IMPORTANT: This migration is MySQL-compatible by default.
-- For PostgreSQL: After running this migration, execute the PostgreSQL-specific
-- trigger creation script (see comments below or create a separate migration).

-- ============================================================================
-- USERS TABLE
-- ============================================================================
-- Note: Argon2 hashes include the salt in the hash string itself.
-- The salt column is included for compatibility, but Argon2 stores salt in the hash format:
-- $argon2id$v=19$m=19456,t=2,p=1$salt$hash

-- MySQL version (with ON UPDATE CURRENT_TIMESTAMP)
CREATE TABLE IF NOT EXISTS users (
    uuid CHAR(36) NOT NULL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    salt VARCHAR(255), -- Optional: Argon2 includes salt in hash string
    prename VARCHAR(255) NOT NULL,
    lastname VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    mail_verified INTEGER NOT NULL DEFAULT 0,
    activated INTEGER NOT NULL DEFAULT 1
);

-- PostgreSQL version (if using PostgreSQL, comment out the MySQL table above and use this):
-- CREATE TABLE IF NOT EXISTS users (
--     uuid CHAR(36) NOT NULL PRIMARY KEY,
--     email VARCHAR(255) NOT NULL UNIQUE,
--     password_hash TEXT NOT NULL,
--     salt VARCHAR(255),
--     prename VARCHAR(255) NOT NULL,
--     lastname VARCHAR(255),
--     created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
--     updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
--     mail_verified INTEGER NOT NULL DEFAULT 0,
--     activated INTEGER NOT NULL DEFAULT 1
-- );
--
-- -- PostgreSQL trigger function and trigger (run these after creating the table):
-- CREATE OR REPLACE FUNCTION update_updated_at_column()
-- RETURNS TRIGGER AS $$
-- BEGIN
--     NEW.updated_at = CURRENT_TIMESTAMP;
--     RETURN NEW;
-- END;
-- $$ language 'plpgsql';
--
-- CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
--     FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- ORGANIZATIONS TABLE
-- ============================================================================

-- MySQL version (with ON UPDATE CURRENT_TIMESTAMP)
CREATE TABLE IF NOT EXISTS organizations (
    uuid CHAR(36) NOT NULL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    owner_user_id CHAR(36) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (owner_user_id) REFERENCES users(uuid) ON DELETE RESTRICT
);

-- PostgreSQL version (if using PostgreSQL, comment out the MySQL table above and use this):
-- CREATE TABLE IF NOT EXISTS organizations (
--     uuid CHAR(36) NOT NULL PRIMARY KEY,
--     name VARCHAR(255) NOT NULL,
--     owner_user_id CHAR(36) NOT NULL,
--     created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
--     updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
--     FOREIGN KEY (owner_user_id) REFERENCES users(uuid) ON DELETE RESTRICT
-- );
--
-- -- PostgreSQL trigger (run this after creating the table):
-- CREATE TRIGGER update_organizations_updated_at BEFORE UPDATE ON organizations
--     FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- ORGANIZATION MEMBERS TABLE (many-to-many)
-- ============================================================================
-- Note: MySQL supports ENUM, PostgreSQL uses CHECK constraint
-- This version uses CHECK constraint which works on both databases

CREATE TABLE IF NOT EXISTS organization_members (
    org_id CHAR(36) NOT NULL,
    user_id CHAR(36) NOT NULL,
    role VARCHAR(20) NOT NULL DEFAULT 'member' CHECK (role IN ('owner', 'admin', 'member')),
    joined_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (org_id, user_id),
    FOREIGN KEY (org_id) REFERENCES organizations(uuid) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(uuid) ON DELETE CASCADE
);

-- MySQL-specific: If you prefer ENUM for MySQL, you can alter the column after creation
-- ALTER TABLE organization_members MODIFY role ENUM('owner', 'admin', 'member') NOT NULL DEFAULT 'member';

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Index on users email for fast lookups
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- Indexes on organization_members for efficient queries
CREATE INDEX IF NOT EXISTS idx_org_members_user ON organization_members(user_id);
CREATE INDEX IF NOT EXISTS idx_org_members_org ON organization_members(org_id);

-- Index on organizations owner for efficient lookups
CREATE INDEX IF NOT EXISTS idx_organizations_owner ON organizations(owner_user_id);
