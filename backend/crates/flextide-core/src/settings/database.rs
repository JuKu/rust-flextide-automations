//! Database operations for organizational settings

use crate::database::{DatabaseError, DatabasePool};
use crate::events::{Event, EventDispatcher, EventPayload};
use serde_json::json;
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

/// Set a string value for an organizational setting
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `organization_uuid` - UUID of the organization
/// * `setting_key` - The name/key of the setting to set
/// * `value` - The value to set (can be None to clear the value)
/// * `dispatcher` - Optional event dispatcher to emit events
/// * `user_uuid` - Optional user UUID who made the change
///
/// # Returns
/// Returns `Ok(())` if the setting was successfully saved, or an error if the setting doesn't exist or database operation fails
///
/// # Errors
/// Returns `SettingsDatabaseError` if:
/// - The setting doesn't exist
/// - Database operation fails
pub async fn set_organizational_setting_value(
    pool: &DatabasePool,
    organization_uuid: &str,
    setting_key: &str,
    value: Option<&str>,
    dispatcher: &EventDispatcher,
    user_uuid: Option<&str>,
) -> Result<(), SettingsDatabaseError> {
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

    // Get old value before updating (for event payload)
    let old_value = get_organizational_setting_value(pool, organization_uuid, setting_key).await?;

    // Upsert the setting value
    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query(
                "INSERT INTO organizational_settings_values (organization_uuid, setting_name, value)
                 VALUES (?, ?, ?)
                 ON DUPLICATE KEY UPDATE value = ?",
            )
            .bind(organization_uuid)
            .bind(setting_key)
            .bind(value)
            .bind(value)
            .execute(p)
            .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO organizational_settings_values (organization_uuid, setting_name, value)
                 VALUES ($1, $2, $3)
                 ON CONFLICT (organization_uuid, setting_name) DO UPDATE SET value = $3",
            )
            .bind(organization_uuid)
            .bind(setting_key)
            .bind(value)
            .execute(p)
            .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO organizational_settings_values (organization_uuid, setting_name, value)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(organization_uuid, setting_name) DO UPDATE SET value = ?3",
            )
            .bind(organization_uuid)
            .bind(setting_key)
            .bind(value)
            .execute(p)
            .await?;
        }
    }

    // Emit event after successful save
    let mut event = Event::new(
        "core_setting_updated",
        EventPayload::new(json!({
            "entity_type": "setting",
            "entity_id": setting_key,
            "organization_uuid": organization_uuid,
            "data": {
                "setting_key": setting_key,
                "old_value": old_value,
                "new_value": value
            }
        })),
    )
    .with_organization(organization_uuid);
    
    if let Some(user_uuid) = user_uuid {
        event = event.with_user(user_uuid);
    }
    
    dispatcher.emit(event).await;

    Ok(())
}

/// Set multiple organizational setting values at once
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `organization_uuid` - UUID of the organization
/// * `settings` - A map of setting keys to values (None values will clear the setting)
/// * `dispatcher` - Optional event dispatcher to emit events
/// * `user_uuid` - Optional user UUID who made the change
///
/// # Returns
/// Returns `Ok(())` if all settings were successfully saved, or an error if any setting doesn't exist or database operation fails
///
/// # Errors
/// Returns `SettingsDatabaseError` if:
/// - Any setting doesn't exist
/// - Database operation fails
pub async fn set_organizational_setting_values(
    pool: &DatabasePool,
    organization_uuid: &str,
    settings: &std::collections::HashMap<String, Option<String>>,
    dispatcher: &EventDispatcher,
    user_uuid: Option<&str>,
) -> Result<(), SettingsDatabaseError> {
    // Validate all settings exist before making any changes
    for setting_key in settings.keys() {
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
            return Err(SettingsDatabaseError::SettingNotFound(setting_key.clone()));
        }
    }

    // Get old values before updating (for event payloads)
    let mut old_values = std::collections::HashMap::new();
    for setting_key in settings.keys() {
        let old_value = get_organizational_setting_value(pool, organization_uuid, setting_key).await?;
        old_values.insert(setting_key.clone(), old_value);
    }

    // Update all settings in a transaction-like manner
    // For simplicity, we'll do them one by one, but in a real scenario you might want to batch them
    for (setting_key, value) in settings {
        match pool {
            DatabasePool::MySql(p) => {
                sqlx::query(
                    "INSERT INTO organizational_settings_values (organization_uuid, setting_name, value)
                     VALUES (?, ?, ?)
                     ON DUPLICATE KEY UPDATE value = ?",
                )
                .bind(organization_uuid)
                .bind(setting_key)
                .bind(value.as_deref())
                .bind(value.as_deref())
                .execute(p)
                .await?;
            }
            DatabasePool::Postgres(p) => {
                sqlx::query(
                    "INSERT INTO organizational_settings_values (organization_uuid, setting_name, value)
                     VALUES ($1, $2, $3)
                     ON CONFLICT (organization_uuid, setting_name) DO UPDATE SET value = $3",
                )
                .bind(organization_uuid)
                .bind(setting_key)
                .bind(value.as_deref())
                .execute(p)
                .await?;
            }
            DatabasePool::Sqlite(p) => {
                sqlx::query(
                    "INSERT INTO organizational_settings_values (organization_uuid, setting_name, value)
                     VALUES (?1, ?2, ?3)
                     ON CONFLICT(organization_uuid, setting_name) DO UPDATE SET value = ?3",
                )
                .bind(organization_uuid)
                .bind(setting_key)
                .bind(value.as_deref())
                .execute(p)
                .await?;
            }
        }

        // Emit event for each setting that was updated
        let old_value = old_values.get(setting_key).and_then(|v| v.as_ref());
        let mut event = Event::new(
            "core_setting_updated",
            EventPayload::new(json!({
                "entity_type": "setting",
                "entity_id": setting_key,
                "organization_uuid": organization_uuid,
                "data": {
                    "setting_key": setting_key,
                    "old_value": old_value,
                    "new_value": value.as_deref()
                }
            })),
        )
        .with_organization(organization_uuid);
        
        if let Some(user_uuid) = user_uuid {
            event = event.with_user(user_uuid);
        }
        
        dispatcher.emit(event).await;
    }

    Ok(())
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

