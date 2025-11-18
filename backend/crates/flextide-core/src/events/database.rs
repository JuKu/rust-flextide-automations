//! Database operations for event subscriptions
//!
//! Handles loading and managing event subscriptions stored in the database.

use crate::database::DatabasePool;
use crate::events::subscriber::DatabaseEventSubscription;
use sqlx::Row;

/// Load all active event subscriptions from the database
pub async fn load_event_subscriptions(
    pool: &DatabasePool,
) -> Result<Vec<DatabaseEventSubscription>, sqlx::Error> {
    match pool {
        DatabasePool::MySql(p) => {
            let rows = sqlx::query(
                "SELECT id, event_name, subscriber_type, config, active, organization_uuid, created_from 
                 FROM event_subscriptions 
                 WHERE active = 1 
                 ORDER BY event_name, id"
            )
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .filter_map(|row| {
                    // MySQL JSON type can be read as serde_json::Value directly
                    let config: serde_json::Value = row.try_get("config").ok()?;
                    
                    Some(DatabaseEventSubscription {
                        id: row.get("id"),
                        event_name: row.get("event_name"),
                        subscriber_type: row.get("subscriber_type"),
                        config,
                        active: row.get("active"),
                        organization_uuid: row.try_get("organization_uuid").ok().flatten(),
                        created_from: row.get("created_from"),
                    })
                })
                .collect())
        }
        DatabasePool::Postgres(p) => {
            let rows = sqlx::query(
                "SELECT id, event_name, subscriber_type, config, active, organization_uuid, created_from 
                 FROM event_subscriptions 
                 WHERE active = true 
                 ORDER BY event_name, id"
            )
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .filter_map(|row| {
                    let config: serde_json::Value = row.try_get("config").ok()?;
                    
                    Some(DatabaseEventSubscription {
                        id: row.get("id"),
                        event_name: row.get("event_name"),
                        subscriber_type: row.get("subscriber_type"),
                        config,
                        active: row.get("active"),
                        organization_uuid: row.try_get("organization_uuid").ok().flatten(),
                        created_from: row.get("created_from"),
                    })
                })
                .collect())
        }
        DatabasePool::Sqlite(p) => {
            let rows = sqlx::query(
                "SELECT id, event_name, subscriber_type, config, active, organization_uuid, created_from 
                 FROM event_subscriptions 
                 WHERE active = 1 
                 ORDER BY event_name, id"
            )
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .filter_map(|row| {
                    let config_str: String = row.try_get("config").ok()?;
                    let config = serde_json::from_str(&config_str).unwrap_or_default();
                    
                    Some(DatabaseEventSubscription {
                        id: row.get("id"),
                        event_name: row.get("event_name"),
                        subscriber_type: row.get("subscriber_type"),
                        config,
                        active: row.get("active"),
                        organization_uuid: row.try_get("organization_uuid").ok().flatten(),
                        created_from: row.get("created_from"),
                    })
                })
                .collect())
        }
    }
}

/// Create a new event subscription in the database
#[allow(dead_code)] // Public API - will be used by other modules
pub async fn create_event_subscription(
    pool: &DatabasePool,
    subscription: &DatabaseEventSubscription,
) -> Result<(), sqlx::Error> {
    match pool {
        DatabasePool::MySql(p) => {
            // MySQL JSON type accepts serde_json::Value directly
            sqlx::query(
                "INSERT INTO event_subscriptions 
                 (id, event_name, subscriber_type, config, active, organization_uuid, created_from) 
                 VALUES (?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&subscription.id)
            .bind(&subscription.event_name)
            .bind(&subscription.subscriber_type)
            .bind(&subscription.config)
            .bind(subscription.active)
            .bind(&subscription.organization_uuid)
            .bind(&subscription.created_from)
            .execute(p)
            .await?;
        }
        DatabasePool::Postgres(p) => {
            // PostgreSQL JSON type accepts serde_json::Value directly
            sqlx::query(
                "INSERT INTO event_subscriptions 
                 (id, event_name, subscriber_type, config, active, organization_uuid, created_from) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7)"
            )
            .bind(&subscription.id)
            .bind(&subscription.event_name)
            .bind(&subscription.subscriber_type)
            .bind(&subscription.config)
            .bind(subscription.active)
            .bind(&subscription.organization_uuid)
            .bind(&subscription.created_from)
            .execute(p)
            .await?;
        }
        DatabasePool::Sqlite(p) => {
            // SQLite stores JSON as TEXT, so we need to serialize to string
            let config_json = serde_json::to_string(&subscription.config).unwrap_or_default();
            sqlx::query(
                "INSERT INTO event_subscriptions 
                 (id, event_name, subscriber_type, config, active, organization_uuid, created_from) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
            )
            .bind(&subscription.id)
            .bind(&subscription.event_name)
            .bind(&subscription.subscriber_type)
            .bind(&config_json)
            .bind(subscription.active)
            .bind(&subscription.organization_uuid)
            .bind(&subscription.created_from)
            .execute(p)
            .await?;
        }
    }

    Ok(())
}

/// Delete event subscriptions created by a specific source (e.g., plugin)
#[allow(dead_code)] // Public API - will be used by other modules
pub async fn delete_subscriptions_by_source(
    pool: &DatabasePool,
    created_from: &str,
) -> Result<(), sqlx::Error> {
    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query("DELETE FROM event_subscriptions WHERE created_from = ?")
                .bind(created_from)
                .execute(p)
                .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query("DELETE FROM event_subscriptions WHERE created_from = $1")
                .bind(created_from)
                .execute(p)
                .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query("DELETE FROM event_subscriptions WHERE created_from = ?1")
                .bind(created_from)
                .execute(p)
                .await?;
        }
    }

    Ok(())
}

