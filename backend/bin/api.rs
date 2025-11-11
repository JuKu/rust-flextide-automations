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

    // JWT secret (in production, use environment variable)
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key-change-in-production".to_string());

    let app_state = AppState {
        jwt_secret,
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

