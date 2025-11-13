use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
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

    tracing::info!("Flextide Worker starting...");

    // TODO: Implement worker main loop (poll queue, execute workflows, etc.)
    // For now, keep the process alive
    tokio::signal::ctrl_c().await?;
    tracing::info!("Worker shutting down...");

    Ok(())
}