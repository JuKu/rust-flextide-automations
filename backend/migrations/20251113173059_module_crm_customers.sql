-- Create CRM module tables: customers and customer notes
-- Supports both MySQL and PostgreSQL
-- Note: updated_at is managed by application code, not database triggers

-- ============================================================================
-- MODULE_CRM_CUSTOMERS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS module_crm_customers (
    uuid CHAR(36) NOT NULL PRIMARY KEY,
    organization_uuid CHAR(36) NOT NULL,
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    phone_number VARCHAR(50),
    user_id CHAR(36),
    salutation VARCHAR(10),
    job_title VARCHAR(255),
    department VARCHAR(255),
    company_name VARCHAR(255),
    fax_number VARCHAR(50),
    website_url VARCHAR(500),
    gender VARCHAR(20),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (organization_uuid) REFERENCES organizations(uuid) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(uuid) ON DELETE SET NULL
);

-- ============================================================================
-- MODULE_CRM_CUSTOMER_NOTES TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS module_crm_customer_notes (
    uuid CHAR(36) NOT NULL PRIMARY KEY,
    customer_uuid CHAR(36) NOT NULL,
    note_text TEXT NOT NULL,
    author_id CHAR(36) NOT NULL,
    visible_to_customer INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (customer_uuid) REFERENCES module_crm_customers(uuid) ON DELETE CASCADE,
    FOREIGN KEY (author_id) REFERENCES users(uuid) ON DELETE RESTRICT
);

-- ============================================================================
-- MODULE_CRM_CUSTOMER_ADDRESSES TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS module_crm_customer_addresses (
    uuid CHAR(36) NOT NULL PRIMARY KEY,
    customer_uuid CHAR(36) NOT NULL,
    address_type VARCHAR(50) NOT NULL,
    street VARCHAR(255),
    city VARCHAR(255),
    state_province VARCHAR(255),
    postal_code VARCHAR(50),
    country VARCHAR(100),
    is_primary INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (customer_uuid) REFERENCES module_crm_customers(uuid) ON DELETE CASCADE
);

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Indexes on module_crm_customers for efficient queries
CREATE INDEX IF NOT EXISTS idx_module_crm_customers_org ON module_crm_customers(organization_uuid);
CREATE INDEX IF NOT EXISTS idx_module_crm_customers_email ON module_crm_customers(email);
CREATE INDEX IF NOT EXISTS idx_module_crm_customers_user_id ON module_crm_customers(user_id);
CREATE INDEX IF NOT EXISTS idx_module_crm_customers_company ON module_crm_customers(company_name);
CREATE INDEX IF NOT EXISTS idx_module_crm_customers_created_at ON module_crm_customers(created_at);

-- Indexes on module_crm_customer_notes for efficient queries
CREATE INDEX IF NOT EXISTS idx_module_crm_customer_notes_customer ON module_crm_customer_notes(customer_uuid);
CREATE INDEX IF NOT EXISTS idx_module_crm_customer_notes_author ON module_crm_customer_notes(author_id);
CREATE INDEX IF NOT EXISTS idx_module_crm_customer_notes_created_at ON module_crm_customer_notes(created_at);

-- Indexes on module_crm_customer_addresses for efficient queries
CREATE INDEX IF NOT EXISTS idx_module_crm_customer_addresses_customer ON module_crm_customer_addresses(customer_uuid);
CREATE INDEX IF NOT EXISTS idx_module_crm_customer_addresses_type ON module_crm_customer_addresses(address_type);
CREATE INDEX IF NOT EXISTS idx_module_crm_customer_addresses_primary ON module_crm_customer_addresses(is_primary);

