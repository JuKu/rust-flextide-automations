use api::{create_app, AppState};

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
    
    // Ensure default admin user exists for tests
    flextide_core::user::ensure_default_admin_user(&db_pool)
        .await
        .expect("Failed to create default admin user");
    
    let app_state = AppState {
        jwt_secret,
        db_pool,
    };
    create_app(app_state)
}
