//! Backup execution logic
//!
//! Handles the actual backup creation, including:
//! - Listing database tables
//! - Reading table structures
//! - Reading table data
//! - Writing backup files

use crate::backup::error::BackupError;
use crate::database::DatabasePool;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Column, Row};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Table column information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub default_value: Option<String>,
}

/// Table structure information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableStructure {
    pub name: String,
    pub columns: Vec<TableColumn>,
}

/// Backup file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupFile {
    pub version: String,
    pub created_at: String,
    pub database_type: String,
    pub structure: HashMap<String, TableStructure>,
    pub data: HashMap<String, Vec<Value>>,
}

/// List all tables in the database, filtering out tables starting with `_`
///
/// # Arguments
/// * `pool` - Database connection pool
///
/// # Returns
/// Vector of table names
pub async fn list_tables(pool: &DatabasePool) -> Result<Vec<String>, BackupError> {
    match pool {
        DatabasePool::MySql(p) => {
            // First, get the current database name
            let db_name_row = sqlx::query("SELECT DATABASE() as db_name")
                .fetch_optional(p)
                .await?;
            
            let db_name = db_name_row
                .and_then(|row| row.try_get::<Option<String>, _>("db_name").ok())
                .flatten();
            
            if let Some(db) = &db_name {
                tracing::debug!("Querying tables for database: {}", db);
            } else {
                tracing::warn!("Could not determine current database name");
            }
            
            // First, let's see ALL tables (including those starting with _) for debugging
            let all_tables_query = if let Some(db) = &db_name {
                sqlx::query(
                    "SELECT TABLE_NAME 
                     FROM INFORMATION_SCHEMA.TABLES 
                     WHERE TABLE_SCHEMA = ?
                     ORDER BY TABLE_NAME",
                )
                .bind(db)
            } else {
                sqlx::query(
                    "SELECT TABLE_NAME 
                     FROM INFORMATION_SCHEMA.TABLES 
                     WHERE TABLE_SCHEMA = SCHEMA()
                     ORDER BY TABLE_NAME",
                )
            };
            
            let all_tables_rows = all_tables_query.fetch_all(p).await?;
            let all_tables: Vec<String> = all_tables_rows
                .into_iter()
                .map(|row| row.get::<String, _>("TABLE_NAME"))
                .collect();
            
            tracing::info!("Found {} total tables in database '{}' (including system tables): {:?}", 
                all_tables.len(), 
                db_name.as_deref().unwrap_or("unknown"),
                all_tables
            );
            
            // Now filter out tables starting with _
            // Note: In SQL LIKE, _ is a wildcard, so we need to escape it or use a different approach
            // We'll use SUBSTRING to check the first character, or filter in Rust
            let query = if let Some(db) = &db_name {
                sqlx::query(
                    "SELECT TABLE_NAME 
                     FROM INFORMATION_SCHEMA.TABLES 
                     WHERE TABLE_SCHEMA = ?
                     AND SUBSTRING(TABLE_NAME, 1, 1) != '_'
                     ORDER BY TABLE_NAME",
                )
                .bind(db)
            } else {
                // Fallback: try without schema filter
                sqlx::query(
                    "SELECT TABLE_NAME 
                     FROM INFORMATION_SCHEMA.TABLES 
                     WHERE TABLE_SCHEMA = SCHEMA()
                     AND SUBSTRING(TABLE_NAME, 1, 1) != '_'
                     ORDER BY TABLE_NAME",
                )
            };
            
            let rows = query.fetch_all(p).await?;
            
            let tables: Vec<String> = rows
                .into_iter()
                .map(|row| row.get::<String, _>("TABLE_NAME"))
                .collect();
            
            tracing::info!("Found {} tables after filtering (excluding tables starting with _)", tables.len());
            
            Ok(tables)
        }
        DatabasePool::Postgres(p) => {
            let rows = sqlx::query(
                "SELECT table_name 
                 FROM information_schema.tables 
                 WHERE table_schema = 'public' 
                 AND SUBSTRING(table_name, 1, 1) != '_'
                 AND table_type = 'BASE TABLE'
                 ORDER BY table_name",
            )
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .map(|row| row.get::<String, _>("table_name"))
                .collect())
        }
        DatabasePool::Sqlite(p) => {
            let rows = sqlx::query(
                "SELECT name 
                 FROM sqlite_master 
                 WHERE type = 'table' 
                 AND SUBSTR(name, 1, 1) != '_'
                 ORDER BY name",
            )
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .map(|row| row.get::<String, _>("name"))
                .collect())
        }
    }
}

/// Get the structure of a table
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `table_name` - Name of the table
///
/// # Returns
/// Table structure information
pub async fn get_table_structure(
    pool: &DatabasePool,
    table_name: &str,
) -> Result<TableStructure, BackupError> {
    let columns = match pool {
        DatabasePool::MySql(p) => {
            let rows = sqlx::query(
                "SELECT 
                    COLUMN_NAME,
                    DATA_TYPE,
                    IS_NULLABLE,
                    COLUMN_KEY,
                    COLUMN_DEFAULT
                 FROM INFORMATION_SCHEMA.COLUMNS
                 WHERE TABLE_SCHEMA = DATABASE()
                 AND TABLE_NAME = ?
                 ORDER BY ORDINAL_POSITION",
            )
            .bind(table_name)
            .fetch_all(p)
            .await?;

            rows.into_iter()
                .map(|row| {
                    let column_key: Option<String> = row.get("COLUMN_KEY");
                    TableColumn {
                        name: row.get("COLUMN_NAME"),
                        data_type: row.get("DATA_TYPE"),
                        is_nullable: row.get::<String, _>("IS_NULLABLE") == "YES",
                        is_primary_key: column_key.as_deref() == Some("PRI"),
                        default_value: row.get("COLUMN_DEFAULT"),
                    }
                })
                .collect()
        }
        DatabasePool::Postgres(p) => {
            let rows = sqlx::query(
                "SELECT 
                    column_name,
                    data_type,
                    is_nullable,
                    column_default
                 FROM information_schema.columns
                 WHERE table_schema = 'public'
                 AND table_name = $1
                 ORDER BY ordinal_position",
            )
            .bind(table_name)
            .fetch_all(p)
            .await?;

            // Get primary key columns separately
            let pk_rows = sqlx::query(
                "SELECT column_name
                 FROM information_schema.table_constraints tc
                 JOIN information_schema.key_column_usage kcu
                 ON tc.constraint_name = kcu.constraint_name
                 WHERE tc.table_schema = 'public'
                 AND tc.table_name = $1
                 AND tc.constraint_type = 'PRIMARY KEY'",
            )
            .bind(table_name)
            .fetch_all(p)
            .await?;

            let pk_columns: std::collections::HashSet<String> = pk_rows
                .into_iter()
                .map(|row| row.get::<String, _>("column_name"))
                .collect();

            rows.into_iter()
                .map(|row| {
                    let col_name: String = row.get("column_name");
                    TableColumn {
                        name: col_name.clone(),
                        data_type: row.get("data_type"),
                        is_nullable: row.get::<String, _>("is_nullable") == "YES",
                        is_primary_key: pk_columns.contains(&col_name),
                        default_value: row.get("column_default"),
                    }
                })
                .collect()
        }
        DatabasePool::Sqlite(p) => {
            let rows = sqlx::query(
                "SELECT 
                    name,
                    type,
                    \"notnull\",
                    dflt_value,
                    pk
                 FROM pragma_table_info(?)
                 ORDER BY cid",
            )
            .bind(table_name)
            .fetch_all(p)
            .await?;

            rows.into_iter()
                .map(|row| TableColumn {
                    name: row.get("name"),
                    data_type: row.get("type"),
                    is_nullable: row.get::<i32, _>("notnull") == 0,
                    is_primary_key: row.get::<i32, _>("pk") != 0,
                    default_value: row.get("dflt_value"),
                })
                .collect()
        }
    };

    Ok(TableStructure {
        name: table_name.to_string(),
        columns,
    })
}

/// Helper function to extract a value from a row column as JSON Value
fn extract_value_from_row<R: sqlx::Row>(row: &R, col_index: usize, _col_name: &str) -> Value 
where
    usize: sqlx::ColumnIndex<R>,
    for<'r> String: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    for<'r> Option<String>: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    for<'r> i64: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    for<'r> Option<i64>: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    for<'r> f64: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    for<'r> Option<f64>: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    for<'r> bool: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    for<'r> Option<bool>: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    for<'r> Value: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    for<'r> Option<Value>: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
{
    // Try JSON/JSONB first (for MySQL JSON, PostgreSQL JSONB)
    if let Ok(Some(v)) = row.try_get::<Option<Value>, _>(col_index) {
        return v;
    }
    
    // Try as string (covers most types)
    if let Ok(Some(v)) = row.try_get::<Option<String>, _>(col_index) {
        // Try to parse as JSON if it looks like JSON
        if v.trim_start().starts_with('{') || v.trim_start().starts_with('[') {
            if let Ok(json_val) = serde_json::from_str::<Value>(&v) {
                return json_val;
            }
        }
        return Value::String(v);
    }
    
    // Try as integer
    if let Ok(Some(v)) = row.try_get::<Option<i64>, _>(col_index) {
        return Value::Number(v.into());
    }
    
    // Try as float
    if let Ok(Some(v)) = row.try_get::<Option<f64>, _>(col_index) {
        if let Some(num) = serde_json::Number::from_f64(v) {
            return Value::Number(num);
        }
    }
    
    // Try as boolean
    if let Ok(Some(v)) = row.try_get::<Option<bool>, _>(col_index) {
        return Value::Bool(v);
    }
    
    // Check if NULL
    if let Ok(None) = row.try_get::<Option<String>, _>(col_index) {
        return Value::Null;
    }
    
    // Fallback: try to get as string anyway
    if let Ok(v) = row.try_get::<String, _>(col_index) {
        return Value::String(v);
    }
    
    // Last resort: return null
    Value::Null
}

/// Get all data from a table as JSON
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `table_name` - Name of the table
///
/// # Returns
/// Vector of JSON objects representing rows
pub async fn get_table_data(
    pool: &DatabasePool,
    table_name: &str,
) -> Result<Vec<Value>, BackupError> {
    // Validate table name to prevent SQL injection
    // Table names should only contain alphanumeric characters and underscores
    if !table_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(BackupError::BackupExecutionFailed(
            format!("Invalid table name: {}", table_name)
        ));
    }
    
    // Use database-specific quoting for table names
    let query = match pool {
        DatabasePool::MySql(_) => format!("SELECT * FROM `{}`", table_name),
        DatabasePool::Postgres(_) => format!(r#"SELECT * FROM "{}""#, table_name),
        DatabasePool::Sqlite(_) => format!(r#"SELECT * FROM "{}""#, table_name),
    };
    
    match pool {
        DatabasePool::MySql(p) => {
            // Safe because table_name is validated above
            let rows = sqlx::query(&query).fetch_all(p).await?;
            
            let mut result = Vec::new();
            for row in rows {
                let mut row_map = serde_json::Map::new();
                for (i, column) in row.columns().iter().enumerate() {
                    let value = extract_value_from_row(&row, i, column.name());
                    row_map.insert(column.name().to_string(), value);
                }
                result.push(Value::Object(row_map));
            }
            Ok(result)
        }
        DatabasePool::Postgres(p) => {
            // Safe because table_name is validated above
            let rows = sqlx::query(&query).fetch_all(p).await?;
            
            let mut result = Vec::new();
            for row in rows {
                let mut row_map = serde_json::Map::new();
                for (i, column) in row.columns().iter().enumerate() {
                    let value = extract_value_from_row(&row, i, column.name());
                    row_map.insert(column.name().to_string(), value);
                }
                result.push(Value::Object(row_map));
            }
            Ok(result)
        }
        DatabasePool::Sqlite(p) => {
            // Safe because table_name is validated above
            let rows = sqlx::query(&query).fetch_all(p).await?;
            
            let mut result = Vec::new();
            for row in rows {
                let mut row_map = serde_json::Map::new();
                for (i, column) in row.columns().iter().enumerate() {
                    let value = extract_value_from_row(&row, i, column.name());
                    row_map.insert(column.name().to_string(), value);
                }
                result.push(Value::Object(row_map));
            }
            Ok(result)
        }
    }
}

/// Execute a backup by creating a JSON backup file
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `backup_uuid` - UUID of the backup record
/// * `backup_path` - Directory where backup files are stored
///
/// # Returns
/// Path to the created backup file
pub async fn execute_backup(
    pool: &DatabasePool,
    backup_uuid: &str,
    backup_path: &str,
) -> Result<PathBuf, BackupError> {
    use crate::backup::database;
    
    tracing::info!("Starting backup execution for backup UUID: {}", backup_uuid);
    
    // Get backup record to find filename
    let backup = get_backup_by_uuid(pool, backup_uuid).await?;
    tracing::debug!("Retrieved backup record: filename={}, status={:?}", backup.filename, backup.backup_status);
    
    // Ensure backup directory exists
    let backup_dir = Path::new(backup_path);
    if !backup_dir.exists() {
        tracing::info!("Backup directory does not exist, creating: {}", backup_path);
        fs::create_dir_all(backup_dir)
            .map_err(|e| {
                tracing::error!("Failed to create backup directory {}: {}", backup_path, e);
                BackupError::BackupExecutionFailed(format!("Failed to create backup directory: {}", e))
            })?;
        tracing::info!("Successfully created backup directory: {}", backup_path);
    } else {
        tracing::debug!("Backup directory already exists: {}", backup_path);
    }
    
    // Get database type
    let db_type = match pool {
        DatabasePool::MySql(_) => "mysql",
        DatabasePool::Postgres(_) => "postgresql",
        DatabasePool::Sqlite(_) => "sqlite",
    };
    tracing::info!("Database type: {}", db_type);
    
    // List all tables (excluding those starting with _)
    tracing::info!("Listing database tables...");
    let tables = list_tables(pool).await?;
    tracing::info!("Found {} tables to backup", tables.len());
    if tables.is_empty() {
        tracing::warn!("No tables found to backup! This might indicate an issue with the database query or all tables are filtered out.");
    } else {
        tracing::info!("Tables to backup:");
        for table in &tables {
            tracing::info!("  - {}", table);
        }
    }
    
    // Build structure and data
    let mut structure_map = HashMap::new();
    let mut data_map = HashMap::new();
    
    for table_name in &tables {
        tracing::debug!("Processing table: {}", table_name);
        
        // Get table structure
        let structure = get_table_structure(pool, table_name).await?;
        structure_map.insert(table_name.clone(), structure);
        tracing::debug!("  - Retrieved structure for table: {} ({} columns)", table_name, structure_map[table_name].columns.len());
        
        // Get table data
        let data = get_table_data(pool, table_name).await?;
        let row_count = data.len();
        data_map.insert(table_name.clone(), data);
        tracing::debug!("  - Retrieved {} rows from table: {}", row_count, table_name);
    }
    
    tracing::info!("Completed processing all tables. Total tables: {}", tables.len());
    
    // Create backup file structure
    tracing::info!("Creating backup file structure...");
    let backup_file = BackupFile {
        version: "1.0".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        database_type: db_type.to_string(),
        structure: structure_map,
        data: data_map,
    };
    
    // Write to file
    let filename = if backup.filename.ends_with(".json.bkp") {
        backup.filename.clone()
    } else {
        format!("{}.json.bkp", backup.filename)
    };
    
    let file_path = backup_dir.join(&filename);
    tracing::info!("Serializing backup data to JSON...");
    let json_content = serde_json::to_string_pretty(&backup_file)
        .map_err(|e| {
            tracing::error!("Failed to serialize backup to JSON: {}", e);
            BackupError::BackupExecutionFailed(format!("Failed to serialize backup: {}", e))
        })?;
    
    let file_size = json_content.len();
    tracing::info!("Backup JSON size: {} bytes, writing to file: {}", file_size, file_path.display());
    
    fs::write(&file_path, json_content)
        .map_err(|e| {
            tracing::error!("Failed to write backup file {}: {}", file_path.display(), e);
            BackupError::BackupExecutionFailed(format!("Failed to write backup file: {}", e))
        })?;
    
    tracing::info!("Successfully wrote backup file: {} ({} bytes)", file_path.display(), file_size);
    
    // Update full_path and status in database
    let full_path_str = file_path.to_string_lossy().to_string();
    tracing::debug!("Updating backup path in database: {}", full_path_str);
    database::update_backup_path(pool, backup_uuid, &full_path_str).await?;
    
    tracing::info!("Updating backup status to COMPLETED");
    database::update_backup_status(pool, backup_uuid, crate::backup::backup::BackupStatus::Completed).await?;
    
    tracing::info!("Backup execution completed successfully: {} -> {}", backup_uuid, file_path.display());
    
    Ok(file_path)
}

/// Get backup by UUID (internal helper)
pub async fn get_backup_by_uuid(
    pool: &DatabasePool,
    backup_uuid: &str,
) -> Result<crate::backup::backup::Backup, BackupError> {
    match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query(
                "SELECT uuid, filename, full_path, creator_user_uuid, target_location, 
                        job_type, backup_status, backup_hash_checksum, is_encrypted, 
                        encryption_algorithm, encryption_master_key_name, error_json,
                        start_timestamp, created_at
                 FROM backups WHERE uuid = ?",
            )
            .bind(backup_uuid)
            .fetch_optional(p)
            .await?;
            
            let row = row.ok_or(BackupError::BackupNotFound)?;
            Ok(crate::backup::backup::Backup {
                uuid: row.get("uuid"),
                filename: row.get("filename"),
                full_path: row.get("full_path"),
                creator_user_uuid: row.get("creator_user_uuid"),
                target_location: row.get("target_location"),
                job_type: row.get("job_type"),
                backup_status: crate::backup::backup::BackupStatus::from(
                    row.get::<String, _>("backup_status").as_str()
                ),
                backup_hash_checksum: row.get("backup_hash_checksum"),
                is_encrypted: row.get::<i32, _>("is_encrypted") != 0,
                encryption_algorithm: row.get("encryption_algorithm"),
                encryption_master_key_name: row.get("encryption_master_key_name"),
                error_json: row.get::<Option<String>, _>("error_json")
                    .as_ref()
                    .and_then(|s| serde_json::from_str::<Value>(s).ok()),
                start_timestamp: row.get("start_timestamp"),
                created_at: row.get("created_at"),
                file_exists: None,
            })
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT uuid, filename, full_path, creator_user_uuid, target_location,
                        job_type, backup_status, backup_hash_checksum, is_encrypted,
                        encryption_algorithm, encryption_master_key_name, error_json,
                        start_timestamp, created_at
                 FROM backups WHERE uuid = $1",
            )
            .bind(backup_uuid)
            .fetch_optional(p)
            .await?;
            
            let row = row.ok_or(BackupError::BackupNotFound)?;
            Ok(crate::backup::backup::Backup {
                uuid: row.get("uuid"),
                filename: row.get("filename"),
                full_path: row.get("full_path"),
                creator_user_uuid: row.get("creator_user_uuid"),
                target_location: row.get("target_location"),
                job_type: row.get("job_type"),
                backup_status: crate::backup::backup::BackupStatus::from(
                    row.get::<String, _>("backup_status").as_str()
                ),
                backup_hash_checksum: row.get("backup_hash_checksum"),
                is_encrypted: row.get::<i32, _>("is_encrypted") != 0,
                encryption_algorithm: row.get("encryption_algorithm"),
                encryption_master_key_name: row.get("encryption_master_key_name"),
                error_json: row.get::<Option<String>, _>("error_json")
                    .as_ref()
                    .and_then(|s| serde_json::from_str::<Value>(s).ok()),
                start_timestamp: row.get("start_timestamp"),
                created_at: row.get("created_at"),
                file_exists: None,
            })
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT uuid, filename, full_path, creator_user_uuid, target_location,
                        job_type, backup_status, backup_hash_checksum, is_encrypted,
                        encryption_algorithm, encryption_master_key_name, error_json,
                        start_timestamp, created_at
                 FROM backups WHERE uuid = ?1",
            )
            .bind(backup_uuid)
            .fetch_optional(p)
            .await?;
            
            let row = row.ok_or(BackupError::BackupNotFound)?;
            Ok(crate::backup::backup::Backup {
                uuid: row.get("uuid"),
                filename: row.get("filename"),
                full_path: row.get("full_path"),
                creator_user_uuid: row.get("creator_user_uuid"),
                target_location: row.get("target_location"),
                job_type: row.get("job_type"),
                backup_status: crate::backup::backup::BackupStatus::from(
                    row.get::<String, _>("backup_status").as_str()
                ),
                backup_hash_checksum: row.get("backup_hash_checksum"),
                is_encrypted: row.get::<i32, _>("is_encrypted") != 0,
                encryption_algorithm: row.get("encryption_algorithm"),
                encryption_master_key_name: row.get("encryption_master_key_name"),
                error_json: row.get::<Option<String>, _>("error_json")
                    .as_ref()
                    .and_then(|s| serde_json::from_str::<Value>(s).ok()),
                start_timestamp: row.get("start_timestamp"),
                created_at: row.get("created_at"),
                file_exists: None,
            })
        }
    }
}

