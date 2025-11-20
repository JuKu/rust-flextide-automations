//! Database connection and pool management
//!
//! Provides reusable database connection logic for both MySQL and PostgreSQL.
//! Supports connection pooling and automatic database type detection from DATABASE_URL.
//! Automatically loads DATABASE_URL from .env file if not set as environment variable.

use sqlx::{mysql::MySqlPool, postgres::PgPool, sqlite::SqlitePool, Pool};
use thiserror::Error;

/// Get DATABASE_URL from environment variable or .env file
///
/// First checks if DATABASE_URL is set as an environment variable.
/// If not, attempts to load it from a .env file:
/// 1. Tries `backend/.env` (when running from project root)
/// 2. Tries `.env` in current directory (when running from backend directory)
///
/// # Errors
/// Returns `DatabaseError` if DATABASE_URL is not found in either location
fn get_database_url() -> Result<String, DatabaseError> {
    // First, try environment variable (already set)
    if let Ok(url) = std::env::var("DATABASE_URL") {
        return Ok(url);
    }

    // If not found, try loading from .env file
    // Try backend/.env first (when running from project root)
    let _ = dotenvy::from_filename("backend/.env");
    if let Ok(url) = std::env::var("DATABASE_URL") {
        return Ok(url);
    }

    // Try .env in current directory (when running from backend directory)
    let _ = dotenvy::dotenv();
    if let Ok(url) = std::env::var("DATABASE_URL") {
        return Ok(url);
    }

    Err(DatabaseError::MissingDatabaseUrl)
}

/// Database connection errors
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Database URL not found in environment variables or .env file")]
    MissingDatabaseUrl,

    #[error("Failed to parse database URL: {0}")]
    InvalidDatabaseUrl(String),

    #[error("Unsupported database type. Only MySQL, PostgreSQL, and SQLite are supported")]
    UnsupportedDatabaseType,

    #[error("Failed to create database pool: {0}")]
    PoolCreationFailed(#[from] sqlx::Error),

    #[error("Migration failed: {0}")]
    MigrationFailed(#[from] sqlx::migrate::MigrateError),
}

impl From<crate::user::UserDatabaseError> for DatabaseError {
    fn from(err: crate::user::UserDatabaseError) -> Self {
        match err {
            crate::user::UserDatabaseError::Database(e) => e,
            crate::user::UserDatabaseError::Sql(e) => DatabaseError::PoolCreationFailed(e),
            crate::user::UserDatabaseError::UserCreation(e) => {
                // Convert UserCreationError to a database error
                // Since UserCreationError can't be directly converted, we wrap it in PoolCreationFailed
                // with a formatted message
                DatabaseError::PoolCreationFailed(sqlx::Error::Configuration(
                    format!("User creation error: {}", e).into(),
                ))
            }
        }
    }
}

/// Database type detected from connection URL
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    MySql,
    Postgres,
    Sqlite,
}

impl DatabaseType {
    /// Detect database type from connection URL
    pub fn from_url(url: &str) -> Result<Self, DatabaseError> {
        if url.starts_with("mysql://") {
            Ok(DatabaseType::MySql)
        } else if url.starts_with("postgres://") || url.starts_with("postgresql://") {
            Ok(DatabaseType::Postgres)
        } else if url.starts_with("sqlite://") || url.starts_with("sqlite:") {
            Ok(DatabaseType::Sqlite)
        } else {
            Err(DatabaseError::UnsupportedDatabaseType)
        }
    }
}

/// Create a MySQL database pool from DATABASE_URL environment variable or .env file
///
/// # Errors
/// Returns `DatabaseError` if DATABASE_URL is missing, invalid, or pool creation fails
pub async fn create_mysql_pool() -> Result<Pool<sqlx::MySql>, DatabaseError> {
    let database_url = get_database_url()?;

    let pool = MySqlPool::connect(&database_url).await?;
    Ok(pool)
}

/// Create a PostgreSQL database pool from DATABASE_URL environment variable or .env file
///
/// # Errors
/// Returns `DatabaseError` if DATABASE_URL is missing, invalid, or pool creation fails
pub async fn create_postgres_pool() -> Result<Pool<sqlx::Postgres>, DatabaseError> {
    let database_url = get_database_url()?;

    let pool = PgPool::connect(&database_url).await?;
    Ok(pool)
}

/// Create a database pool with automatic type detection from DATABASE_URL
///
/// Detects whether to use MySQL, PostgreSQL, or SQLite based on the URL scheme.
/// Loads DATABASE_URL from environment variable or .env file.
///
/// # Errors
/// Returns `DatabaseError` if DATABASE_URL is missing, invalid, unsupported, or pool creation fails
pub async fn create_pool() -> Result<DatabasePool, DatabaseError> {
    let database_url = get_database_url()?;

    let db_type = DatabaseType::from_url(&database_url)?;

    match db_type {
        DatabaseType::MySql => {
            let pool = MySqlPool::connect(&database_url).await?;
            Ok(DatabasePool::MySql(pool))
        }
        DatabaseType::Postgres => {
            let pool = PgPool::connect(&database_url).await?;
            Ok(DatabasePool::Postgres(pool))
        }
        DatabaseType::Sqlite => {
            let pool = SqlitePool::connect(&database_url).await?;
            Ok(DatabasePool::Sqlite(pool))
        }
    }
}

/// Create an in-memory SQLite database pool for testing
///
/// This creates a temporary SQLite database that exists only in memory.
/// Perfect for unit tests that don't need persistent data.
///
/// # Errors
/// Returns `DatabaseError` if pool creation fails
pub async fn create_test_pool() -> Result<DatabasePool, DatabaseError> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    Ok(DatabasePool::Sqlite(pool))
}

/// Enum wrapper for database pools supporting MySQL, PostgreSQL, and SQLite
#[derive(Clone)]
pub enum DatabasePool {
    MySql(Pool<sqlx::MySql>),
    Postgres(Pool<sqlx::Postgres>),
    Sqlite(Pool<sqlx::Sqlite>),
}

impl DatabasePool {
    /// Get the database type of this pool
    pub fn database_type(&self) -> DatabaseType {
        match self {
            DatabasePool::MySql(_) => DatabaseType::MySql,
            DatabasePool::Postgres(_) => DatabaseType::Postgres,
            DatabasePool::Sqlite(_) => DatabaseType::Sqlite,
        }
    }

    /// Run database migrations
    ///
    /// # Arguments
    /// * `migrations_path` - Path to the migrations directory (e.g., "./migrations" or "../migrations")
    ///
    /// # Errors
    /// Returns `DatabaseError` if migration execution fails
    pub async fn run_migrations(&self, migrations_path: &str) -> Result<(), DatabaseError> {
        let path = std::path::Path::new(migrations_path);
        
        // Verify path exists and is a directory
        if !path.exists() {
            let error_msg = format!("Migrations directory does not exist: {:?}", path);
            tracing::error!("{}", error_msg);
            return Err(DatabaseError::MigrationFailed(
                sqlx::migrate::MigrateError::from(sqlx::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    error_msg,
                )))
            ));
        }
        
        if !path.is_dir() {
            let error_msg = format!("Migrations path is not a directory: {:?}", path);
            tracing::error!("{}", error_msg);
            return Err(DatabaseError::MigrationFailed(
                sqlx::migrate::MigrateError::from(sqlx::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    error_msg,
                )))
            ));
        }
        
        // Try to list migration files for better error messages
        let migration_files: Vec<String> = match std::fs::read_dir(path) {
            Ok(entries) => {
                entries
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
                    .collect()
            }
            Err(e) => {
                tracing::warn!("Failed to read migrations directory: {}", e);
                vec![]
            }
        };
        
        if !migration_files.is_empty() {
            tracing::debug!("Found {} migration files in {:?}: {:?}", migration_files.len(), path, migration_files);
        } else {
            tracing::warn!("No migration files found in {:?}", path);
        }
        
        match self {
            DatabasePool::MySql(pool) => {
                tracing::debug!("Creating migrator for MySQL from path: {:?}", path);
                let migrator = sqlx::migrate::Migrator::new(path)
                    .await
                    .map_err(|e| {
                        tracing::error!("Failed to create migrator for MySQL from {:?}: {}", path, e);
                        DatabaseError::MigrationFailed(e)
                    })?;
                
                tracing::debug!("Running MySQL migrations...");
                migrator.run(pool)
                    .await
                    .map_err(|e| {
                        tracing::error!("Failed to run MySQL migrations: {}", e);
                        DatabaseError::MigrationFailed(e)
                    })?;
            }
            DatabasePool::Postgres(pool) => {
                tracing::debug!("Creating migrator for PostgreSQL from path: {:?}", path);
                let migrator = sqlx::migrate::Migrator::new(path)
                    .await
                    .map_err(|e| {
                        tracing::error!("Failed to create migrator for PostgreSQL from {:?}: {}", path, e);
                        DatabaseError::MigrationFailed(e)
                    })?;
                
                tracing::debug!("Running PostgreSQL migrations...");
                migrator.run(pool)
                    .await
                    .map_err(|e| {
                        tracing::error!("Failed to run PostgreSQL migrations: {}", e);
                        DatabaseError::MigrationFailed(e)
                    })?;
            }
            DatabasePool::Sqlite(pool) => {
                tracing::debug!("Creating migrator for SQLite from path: {:?}", path);
                let migrator = sqlx::migrate::Migrator::new(path)
                    .await
                    .map_err(|e| {
                        tracing::error!("Failed to create migrator for SQLite from {:?}: {}", path, e);
                        DatabaseError::MigrationFailed(e)
                    })?;
                
                tracing::debug!("Running SQLite migrations...");
                migrator.run(pool)
                    .await
                    .map_err(|e| {
                        tracing::error!("Failed to run SQLite migrations: {}", e);
                        DatabaseError::MigrationFailed(e)
                    })?;
            }
        }
        Ok(())
    }

    /// Execute a query that works with all database types
    /// 
    /// This is a convenience method for simple queries. For complex queries,
    /// you may need to match on the pool type and use database-specific APIs.
    pub async fn execute(&self, query: &str) -> Result<u64, DatabaseError> {
        match self {
            DatabasePool::MySql(pool) => {
                let result = sqlx::query(query).execute(pool).await?;
                Ok(result.rows_affected())
            }
            DatabasePool::Postgres(pool) => {
                let result = sqlx::query(query).execute(pool).await?;
                Ok(result.rows_affected())
            }
            DatabasePool::Sqlite(pool) => {
                let result = sqlx::query(query).execute(pool).await?;
                Ok(result.rows_affected())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_type_detection() {
        assert_eq!(
            DatabaseType::from_url("mysql://user:pass@localhost/db").unwrap(),
            DatabaseType::MySql
        );
        assert_eq!(
            DatabaseType::from_url("postgres://user:pass@localhost/db").unwrap(),
            DatabaseType::Postgres
        );
        assert_eq!(
            DatabaseType::from_url("postgresql://user:pass@localhost/db").unwrap(),
            DatabaseType::Postgres
        );
        assert_eq!(
            DatabaseType::from_url("sqlite:///path/to/db").unwrap(),
            DatabaseType::Sqlite
        );
        assert_eq!(
            DatabaseType::from_url("sqlite::memory:").unwrap(),
            DatabaseType::Sqlite
        );
        assert!(DatabaseType::from_url("invalid://db").is_err());
    }
}

