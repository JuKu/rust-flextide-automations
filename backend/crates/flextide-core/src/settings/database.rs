//! Database operations for organizational settings

use crate::database::{DatabaseError, DatabasePool};
use sqlx::Row;
use thiserror::Error;

/// Error type for settings database operations
#[derive(Debug, Error)]
pub enum SettingsDatabaseError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("SQL execution error: {0}")]
    Sql(#[from] sqlx::Error),

    #[error("Setting not found: {0}")]
    SettingNotFound(String),

    #[error("Setting value not found for organization: {0}")]
    SettingValueNotFound(String),
}

/// Get a string value for an organizational setting
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `organization_uuid` - UUID of the organization
/// * `setting_key` - The name/key of the setting to retrieve
///
/// # Returns
/// Returns `Ok(Some(String))` if the setting value exists, `Ok(None)` if the setting exists but has no value,
/// or an error if the setting doesn't exist or database operation fails
///
/// # Errors
/// Returns `SettingsDatabaseError` if:
/// - The setting doesn't exist
/// - Database operation fails
pub async fn get_organizational_setting_value(
    pool: &DatabasePool,
    organization_uuid: &str,
    setting_key: &str,
) -> Result<Option<String>, SettingsDatabaseError> {
    // First check if the setting exists
    let setting_exists = match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query(
                "SELECT COUNT(*) as count FROM organizational_settings WHERE name = ?",
            )
            .bind(setting_key)
            .fetch_one(p)
            .await?;
            let count: i64 = row.get("count");
            count > 0
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT COUNT(*) as count FROM organizational_settings WHERE name = $1",
            )
            .bind(setting_key)
            .fetch_one(p)
            .await?;
            let count: i64 = row.get("count");
            count > 0
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT COUNT(*) as count FROM organizational_settings WHERE name = ?1",
            )
            .bind(setting_key)
            .fetch_one(p)
            .await?;
            let count: i64 = row.get("count");
            count > 0
        }
    };

    if !setting_exists {
        return Err(SettingsDatabaseError::SettingNotFound(setting_key.to_string()));
    }

    // Get the setting value for the organization
    let value = match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query(
                "SELECT value FROM organizational_settings_values
                 WHERE organization_uuid = ? AND setting_name = ?",
            )
            .bind(organization_uuid)
            .bind(setting_key)
            .fetch_optional(p)
            .await?;

            row.and_then(|row| row.get::<Option<String>, _>("value"))
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT value FROM organizational_settings_values
                 WHERE organization_uuid = $1 AND setting_name = $2",
            )
            .bind(organization_uuid)
            .bind(setting_key)
            .fetch_optional(p)
            .await?;

            row.and_then(|row| row.get::<Option<String>, _>("value"))
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT value FROM organizational_settings_values
                 WHERE organization_uuid = ?1 AND setting_name = ?2",
            )
            .bind(organization_uuid)
            .bind(setting_key)
            .fetch_optional(p)
            .await?;

            row.and_then(|row| row.get::<Option<String>, _>("value"))
        }
    };

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_database_error_display() {
        let error = SettingsDatabaseError::SettingNotFound("test_setting".to_string());
        assert!(error.to_string().contains("test_setting"));
        assert!(error.to_string().contains("Setting not found"));

        let error = SettingsDatabaseError::SettingValueNotFound("test_org".to_string());
        assert!(error.to_string().contains("test_org"));
        assert!(error.to_string().contains("Setting value not found"));
    }

    #[test]
    fn test_settings_database_error_from_database_error() {
        let db_error = DatabaseError::PoolCreationFailed(sqlx::Error::PoolClosed);
        let settings_error: SettingsDatabaseError = db_error.into();
        match settings_error {
            SettingsDatabaseError::Database(_) => {}
            _ => panic!("Expected Database variant"),
        }
    }

    #[test]
    fn test_settings_database_error_from_sqlx_error() {
        let sql_error = sqlx::Error::RowNotFound;
        let settings_error: SettingsDatabaseError = sql_error.into();
        match settings_error {
            SettingsDatabaseError::Sql(_) => {}
            _ => panic!("Expected Sql variant"),
        }
    }
}

