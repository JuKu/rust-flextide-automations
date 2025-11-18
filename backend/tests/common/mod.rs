use api::{create_app, AppState};

#[allow(dead_code)]
pub async fn create_test_app() -> axum::Router {
    let jwt_secret = "test-secret-key".to_string();
    
    // Use in-memory SQLite database for tests - no real database needed!
    let db_pool = flextide_core::database::create_test_pool()
        .await
        .expect("Failed to create test database pool");
    
    // Create users table for tests (SQLite syntax)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            uuid CHAR(36) NOT NULL PRIMARY KEY,
            email VARCHAR(255) NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            salt VARCHAR(255),
            prename VARCHAR(255) NOT NULL,
            lastname VARCHAR(255),
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            mail_verified INTEGER NOT NULL DEFAULT 0,
            activated INTEGER NOT NULL DEFAULT 1
        )"
    )
    .execute(match &db_pool {
        flextide_core::database::DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to create users table");
    
    // Create organizations table for tests (must be created before organization_members)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS organizations (
            uuid CHAR(36) NOT NULL PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            owner_user_id CHAR(36) NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(match &db_pool {
        flextide_core::database::DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to create organizations table");
    
    // Create organization_members table for tests (must be created before ensure_default_admin_user)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS organization_members (
            org_id CHAR(36) NOT NULL,
            user_id CHAR(36) NOT NULL,
            role VARCHAR(20) NOT NULL DEFAULT 'member',
            joined_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (org_id, user_id)
        )"
    )
    .execute(match &db_pool {
        flextide_core::database::DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to create organization_members table");
    
    // Create permission_groups table for tests (must be created before permissions)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS permission_groups (
            id CHAR(36) NOT NULL PRIMARY KEY,
            name VARCHAR(255) NOT NULL UNIQUE,
            title VARCHAR(255) NOT NULL,
            description TEXT,
            visible INTEGER NOT NULL DEFAULT 1,
            sort_order INTEGER NOT NULL DEFAULT 0
        )"
    )
    .execute(match &db_pool {
        flextide_core::database::DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to create permission_groups table");
    
    // Create permissions table for tests (must be created before user_permissions)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS permissions (
            id CHAR(36) NOT NULL PRIMARY KEY,
            permission_group_name VARCHAR(255) NOT NULL,
            name VARCHAR(255) NOT NULL UNIQUE,
            title VARCHAR(255) NOT NULL,
            description TEXT,
            visible INTEGER NOT NULL DEFAULT 1,
            sort_order INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (permission_group_name) REFERENCES permission_groups(name) ON DELETE RESTRICT
        )"
    )
    .execute(match &db_pool {
        flextide_core::database::DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to create permissions table");
    
    // Create user_permissions table for tests (must be created before ensure_default_admin_user)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS user_permissions (
            user_id CHAR(36) NOT NULL,
            organization_uuid CHAR(36) NOT NULL,
            permission_name VARCHAR(255) NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (user_id, organization_uuid, permission_name),
            FOREIGN KEY (user_id) REFERENCES users(uuid) ON DELETE CASCADE,
            FOREIGN KEY (organization_uuid) REFERENCES organizations(uuid) ON DELETE CASCADE,
            FOREIGN KEY (permission_name) REFERENCES permissions(name) ON DELETE CASCADE
        )"
    )
    .execute(match &db_pool {
        flextide_core::database::DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to create user_permissions table");
    
    // Insert super_admin permission group and permission for tests
    sqlx::query(
        "INSERT OR IGNORE INTO permission_groups (id, name, title, description, visible, sort_order)
         VALUES ('00000000-0000-0000-0000-000000000005', 'super_admin', 'Super Admin', 'Super administrator permissions that grant access to everything in an organization', 1, 0)"
    )
    .execute(match &db_pool {
        flextide_core::database::DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to insert super_admin permission group");
    
    sqlx::query(
        "INSERT OR IGNORE INTO permissions (id, permission_group_name, name, title, description, visible, sort_order)
         VALUES ('20000000-0000-0000-0000-000000000001', 'super_admin', 'super_admin', 'Super Admin', 'Grants the user access to everything in the organization', 1, 1)"
    )
    .execute(match &db_pool {
        flextide_core::database::DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to insert super_admin permission");
    
    // Ensure default admin user exists for tests (must be called after all tables are created)
    flextide_core::user::ensure_default_admin_user(&db_pool)
        .await
        .expect("Failed to create default admin user");
    
    // Create CRM tables for tests
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS module_crm_customers (
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
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(match &db_pool {
        flextide_core::database::DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to create module_crm_customers table");
    
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS module_crm_customer_notes (
            uuid CHAR(36) NOT NULL PRIMARY KEY,
            customer_uuid CHAR(36) NOT NULL,
            note_text TEXT NOT NULL,
            author_id CHAR(36) NOT NULL,
            visible_to_customer INTEGER NOT NULL DEFAULT 0,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(match &db_pool {
        flextide_core::database::DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to create module_crm_customer_notes table");
    
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS module_crm_customer_addresses (
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
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(match &db_pool {
        flextide_core::database::DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to create module_crm_customer_addresses table");
    
    let app_state = AppState {
        jwt_secret,
        db_pool: db_pool.clone(),
    };
    create_app(app_state)
}

/// Create test app and set up a test organization
/// Returns (app, org_uuid, user_uuid, email)
/// This ensures the organization is set up in the same database as the app
#[allow(dead_code)]
pub async fn create_test_app_with_org() -> (axum::Router, String, String, String) {
    let jwt_secret = "test-secret-key".to_string();
    
    // Use in-memory SQLite database for tests - no real database needed!
    let db_pool = flextide_core::database::create_test_pool()
        .await
        .expect("Failed to create test database pool");
    
    // Create users table for tests (SQLite syntax)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            uuid CHAR(36) NOT NULL PRIMARY KEY,
            email VARCHAR(255) NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            salt VARCHAR(255),
            prename VARCHAR(255) NOT NULL,
            lastname VARCHAR(255),
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            mail_verified INTEGER NOT NULL DEFAULT 0,
            activated INTEGER NOT NULL DEFAULT 1
        )"
    )
    .execute(match &db_pool {
        flextide_core::database::DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to create users table");
    
    // Create organizations table for tests (must be created before organization_members)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS organizations (
            uuid CHAR(36) NOT NULL PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            owner_user_id CHAR(36) NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(match &db_pool {
        flextide_core::database::DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to create organizations table");
    
    // Create organization_members table for tests (must be created before ensure_default_admin_user)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS organization_members (
            org_id CHAR(36) NOT NULL,
            user_id CHAR(36) NOT NULL,
            role VARCHAR(20) NOT NULL DEFAULT 'member',
            joined_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (org_id, user_id)
        )"
    )
    .execute(match &db_pool {
        flextide_core::database::DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to create organization_members table");
    
    // Create permission_groups table for tests (must be created before permissions)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS permission_groups (
            id CHAR(36) NOT NULL PRIMARY KEY,
            name VARCHAR(255) NOT NULL UNIQUE,
            title VARCHAR(255) NOT NULL,
            description TEXT,
            visible INTEGER NOT NULL DEFAULT 1,
            sort_order INTEGER NOT NULL DEFAULT 0
        )"
    )
    .execute(match &db_pool {
        flextide_core::database::DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to create permission_groups table");
    
    // Create permissions table for tests (must be created before user_permissions)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS permissions (
            id CHAR(36) NOT NULL PRIMARY KEY,
            permission_group_name VARCHAR(255) NOT NULL,
            name VARCHAR(255) NOT NULL UNIQUE,
            title VARCHAR(255) NOT NULL,
            description TEXT,
            visible INTEGER NOT NULL DEFAULT 1,
            sort_order INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (permission_group_name) REFERENCES permission_groups(name) ON DELETE RESTRICT
        )"
    )
    .execute(match &db_pool {
        flextide_core::database::DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to create permissions table");
    
    // Create user_permissions table for tests (must be created before ensure_default_admin_user)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS user_permissions (
            user_id CHAR(36) NOT NULL,
            organization_uuid CHAR(36) NOT NULL,
            permission_name VARCHAR(255) NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (user_id, organization_uuid, permission_name),
            FOREIGN KEY (user_id) REFERENCES users(uuid) ON DELETE CASCADE,
            FOREIGN KEY (organization_uuid) REFERENCES organizations(uuid) ON DELETE CASCADE,
            FOREIGN KEY (permission_name) REFERENCES permissions(name) ON DELETE CASCADE
        )"
    )
    .execute(match &db_pool {
        flextide_core::database::DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to create user_permissions table");
    
    // Insert super_admin permission group and permission for tests
    sqlx::query(
        "INSERT OR IGNORE INTO permission_groups (id, name, title, description, visible, sort_order)
         VALUES ('00000000-0000-0000-0000-000000000005', 'super_admin', 'Super Admin', 'Super administrator permissions that grant access to everything in an organization', 1, 0)"
    )
    .execute(match &db_pool {
        flextide_core::database::DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to insert super_admin permission group");
    
    sqlx::query(
        "INSERT OR IGNORE INTO permissions (id, permission_group_name, name, title, description, visible, sort_order)
         VALUES ('20000000-0000-0000-0000-000000000001', 'super_admin', 'super_admin', 'Super Admin', 'Grants the user access to everything in the organization', 1, 1)"
    )
    .execute(match &db_pool {
        flextide_core::database::DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to insert super_admin permission");
    
    // Ensure default admin user exists for tests (must be called after all tables are created)
    flextide_core::user::ensure_default_admin_user(&db_pool)
        .await
        .expect("Failed to create default admin user");
    
    // Create CRM tables for tests
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS module_crm_customers (
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
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(match &db_pool {
        flextide_core::database::DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to create module_crm_customers table");
    
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS module_crm_customer_notes (
            uuid CHAR(36) NOT NULL PRIMARY KEY,
            customer_uuid CHAR(36) NOT NULL,
            note_text TEXT NOT NULL,
            author_id CHAR(36) NOT NULL,
            visible_to_customer INTEGER NOT NULL DEFAULT 0,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(match &db_pool {
        flextide_core::database::DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to create module_crm_customer_notes table");
    
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS module_crm_customer_addresses (
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
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(match &db_pool {
        flextide_core::database::DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to create module_crm_customer_addresses table");
    
    // Set up test organization in the same database
    let (org_uuid, user_uuid, email) = setup_test_organization_in_pool(&db_pool).await;
    
    let app_state = AppState {
        jwt_secret,
        db_pool,
    };
    let app = create_app(app_state);
    
    (app, org_uuid, user_uuid, email)
}

/// Helper function to set up test organization and user membership in the test app's database
#[allow(dead_code)]
pub async fn setup_test_organization_in_pool(db_pool: &flextide_core::database::DatabasePool) -> (String, String, String) {
    use flextide_core::database::DatabasePool;
    use uuid::Uuid;
    
    // Get admin user
    let admin_user = flextide_core::user::get_user_by_email(db_pool, "admin@example.com")
        .await
        .expect("Admin user should exist");
    let admin_uuid = admin_user.uuid.clone();
    
    // Create test organization
    let org_uuid = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO organizations (uuid, name, owner_user_id) VALUES (?1, ?2, ?3)"
    )
    .bind(&org_uuid)
    .bind("Test Organization")
    .bind(&admin_uuid)
    .execute(match db_pool {
        DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to create test organization");
    
    // Add admin user to organization
    sqlx::query(
        "INSERT INTO organization_members (org_id, user_id, role) VALUES (?1, ?2, ?3)"
    )
    .bind(&org_uuid)
    .bind(&admin_uuid)
    .bind("owner")
    .execute(match db_pool {
        DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to add user to organization");
    
    // Grant super_admin permission to admin user for the test organization
    sqlx::query(
        "INSERT OR IGNORE INTO user_permissions (user_id, organization_uuid, permission_name)
         VALUES (?1, ?2, 'super_admin')"
    )
    .bind(&admin_uuid)
    .bind(&org_uuid)
    .execute(match db_pool {
        DatabasePool::Sqlite(p) => p,
        _ => unreachable!("Test pool should be SQLite"),
    })
    .await
    .expect("Failed to grant super_admin permission");
    
    (org_uuid, admin_uuid, admin_user.email)
}

