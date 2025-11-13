use api::{create_app, AppState};
use std::net::SocketAddr;
use tokio;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber with default level "info" if RUST_LOG is not set
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize database connection
    tracing::info!("Connecting to database...");
    let db_pool = flextide_core::database::create_pool()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create database pool: {}", e))?;
    tracing::info!("Database connection established (type: {:?})", db_pool.database_type());

    // Run database migrations
    // Try both paths: ./migrations (when running from backend/) and ./backend/migrations (when running from project root)
    tracing::info!("Running database migrations...");
    let migration_result = db_pool.run_migrations("./migrations").await;
    let migration_result = if migration_result.is_err() {
        db_pool.run_migrations("./backend/migrations").await
    } else {
        migration_result
    };
    migration_result.map_err(|e| anyhow::anyhow!("Failed to run database migrations: {}", e))?;
    tracing::info!("Database migrations completed");

    // Ensure default admin user exists
    tracing::info!("Checking for default admin user...");
    flextide_core::user::ensure_default_admin_user(&db_pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to ensure default admin user: {}", e))?;
    tracing::info!("Default admin user ensured (admin@example.com / admin)");

    // JWT secret (in production, use environment variable)
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key-change-in-production".to_string());

    let app_state = AppState {
        jwt_secret,
        db_pool,
    };
    let app = create_app(app_state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Starting API server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("API server is running and ready to accept connections on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}

