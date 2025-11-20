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
    let current_dir = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."));
    tracing::debug!("Current working directory: {:?}", current_dir);
    
    // Check which migrations path exists
    let migrations_paths = vec![
        ("./migrations", current_dir.join("migrations")),
        ("./backend/migrations", current_dir.join("backend").join("migrations")),
    ];
    
    let mut found_path: Option<&str> = None;
    for (path_str, path_buf) in &migrations_paths {
        if path_buf.exists() && path_buf.is_dir() {
            tracing::debug!("Found migrations directory at: {:?} (resolved from: {})", path_buf, path_str);
            found_path = Some(path_str);
            
            // List migration files for debugging
            if let Ok(entries) = std::fs::read_dir(path_buf) {
                let mut migration_files: Vec<String> = entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| {
                        let file_name = e.file_name();
                        let name = file_name.to_string_lossy().to_string();
                        if name.ends_with(".sql") {
                            Some(name)
                        } else {
                            None
                        }
                    })
                    .collect();
                migration_files.sort();
                tracing::debug!("Found {} migration files: {:?}", migration_files.len(), migration_files);
            }
            break;
        } else {
            tracing::warn!("Migrations path does not exist: {:?} (resolved from: {})", path_buf, path_str);
        }
    }
    
    let migrations_path = found_path.ok_or_else(|| {
        anyhow::anyhow!(
            "Migrations directory not found. Tried paths: {:?}. Current directory: {:?}",
            migrations_paths.iter().map(|(s, _)| s).collect::<Vec<_>>(),
            current_dir
        )
    })?;
    
    let migration_result = db_pool.run_migrations(migrations_path).await;
    migration_result.map_err(|e| {
        anyhow::anyhow!(
            "Failed to run database migrations from path '{}': {}. Current directory: {:?}",
            migrations_path,
            e,
            current_dir
        )
    })?;
    tracing::info!("Database migrations completed");

    // Ensure default admin user exists
    tracing::info!("Checking for default admin user...");
    flextide_core::user::ensure_default_admin_user(&db_pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to ensure default admin user: {}", e))?;
    tracing::info!("Default admin user ensured (admin@example.com / admin)");

    // Initialize event dispatcher
    tracing::info!("Initializing event system...");
    let event_dispatcher = flextide_core::events::EventDispatcher::new();
    
    // Load database-backed event subscriptions
    flextide_core::events::initialize(&event_dispatcher, &db_pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize event system: {}", e))?;
    tracing::info!("Event system initialized");

    // JWT secret (in production, use environment variable)
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key-change-in-production".to_string());

    let app_state = AppState {
        jwt_secret,
        db_pool,
        event_dispatcher,
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

