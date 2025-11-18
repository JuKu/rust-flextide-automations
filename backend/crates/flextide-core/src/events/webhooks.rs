//! Webhook support for the event system
//!
//! Provides webhook management and HTTP delivery for events.

use crate::database::DatabasePool;
use crate::events::types::Event;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sha2::Sha256;
use sqlx::Row;
use std::time::Duration;
use tracing::{debug, warn};

type HmacSha256 = Hmac<Sha256>;

/// Webhook configuration stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    pub id: String,
    pub organization_uuid: String,
    pub event_name: String,
    pub url: String,
    pub secret: Option<String>,
    pub headers: Option<JsonValue>,
    pub active: bool,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request structure for creating a new webhook
#[derive(Debug, Deserialize)]
pub struct CreateWebhookRequest {
    pub event_name: String,
    pub url: String,
    pub secret: Option<String>,
    pub headers: Option<JsonValue>,
}

/// Request structure for updating a webhook
#[derive(Debug, Deserialize)]
pub struct UpdateWebhookRequest {
    pub event_name: Option<String>,
    pub url: Option<String>,
    pub secret: Option<String>,
    pub headers: Option<JsonValue>,
    pub active: Option<bool>,
}

/// Load all active webhooks from the database
pub async fn load_webhooks(pool: &DatabasePool) -> Result<Vec<Webhook>, sqlx::Error> {
    match pool {
        DatabasePool::MySql(p) => {
            let rows = sqlx::query(
                "SELECT id, organization_uuid, event_name, url, secret, headers, active, created_by, created_at, updated_at
                 FROM event_webhooks 
                 WHERE active = 1 
                 ORDER BY event_name, id"
            )
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .filter_map(|row| {
                    let headers: Option<JsonValue> = row.try_get("headers").ok().flatten();
                    
                    Some(Webhook {
                        id: row.get("id"),
                        organization_uuid: row.get("organization_uuid"),
                        event_name: row.get("event_name"),
                        url: row.get("url"),
                        secret: row.try_get("secret").ok().flatten(),
                        headers,
                        active: row.get("active"),
                        created_by: row.get("created_by"),
                        created_at: row.get("created_at"),
                        updated_at: row.get("updated_at"),
                    })
                })
                .collect())
        }
        DatabasePool::Postgres(p) => {
            let rows = sqlx::query(
                "SELECT id, organization_uuid, event_name, url, secret, headers, active, created_by, created_at, updated_at
                 FROM event_webhooks 
                 WHERE active = true 
                 ORDER BY event_name, id"
            )
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .filter_map(|row| {
                    let headers: Option<JsonValue> = row.try_get("headers").ok().flatten();
                    
                    Some(Webhook {
                        id: row.get("id"),
                        organization_uuid: row.get("organization_uuid"),
                        event_name: row.get("event_name"),
                        url: row.get("url"),
                        secret: row.try_get("secret").ok().flatten(),
                        headers,
                        active: row.get("active"),
                        created_by: row.get("created_by"),
                        created_at: row.get("created_at"),
                        updated_at: row.get("updated_at"),
                    })
                })
                .collect())
        }
        DatabasePool::Sqlite(p) => {
            let rows = sqlx::query(
                "SELECT id, organization_uuid, event_name, url, secret, headers, active, created_by, created_at, updated_at
                 FROM event_webhooks 
                 WHERE active = 1 
                 ORDER BY event_name, id"
            )
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .filter_map(|row| {
                    let headers_str: Option<String> = row.try_get("headers").ok().flatten();
                    let headers = headers_str.and_then(|s| serde_json::from_str(&s).ok());
                    
                    Some(Webhook {
                        id: row.get("id"),
                        organization_uuid: row.get("organization_uuid"),
                        event_name: row.get("event_name"),
                        url: row.get("url"),
                        secret: row.try_get("secret").ok().flatten(),
                        headers,
                        active: row.get("active"),
                        created_by: row.get("created_by"),
                        created_at: row.get("created_at"),
                        updated_at: row.get("updated_at"),
                    })
                })
                .collect())
        }
    }
}

/// Load webhooks for a specific organization
pub async fn load_webhooks_by_organization(
    pool: &DatabasePool,
    organization_uuid: &str,
) -> Result<Vec<Webhook>, sqlx::Error> {
    match pool {
        DatabasePool::MySql(p) => {
            let rows = sqlx::query(
                "SELECT id, organization_uuid, event_name, url, secret, headers, active, created_by, created_at, updated_at
                 FROM event_webhooks 
                 WHERE organization_uuid = ? AND active = 1
                 ORDER BY event_name, created_at"
            )
            .bind(organization_uuid)
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .filter_map(|row| {
                    let headers: Option<JsonValue> = row.try_get("headers").ok().flatten();
                    
                    Some(Webhook {
                        id: row.get("id"),
                        organization_uuid: row.get("organization_uuid"),
                        event_name: row.get("event_name"),
                        url: row.get("url"),
                        secret: row.try_get("secret").ok().flatten(),
                        headers,
                        active: row.get("active"),
                        created_by: row.get("created_by"),
                        created_at: row.get("created_at"),
                        updated_at: row.get("updated_at"),
                    })
                })
                .collect())
        }
        DatabasePool::Postgres(p) => {
            let rows = sqlx::query(
                "SELECT id, organization_uuid, event_name, url, secret, headers, active, created_by, created_at, updated_at
                 FROM event_webhooks 
                 WHERE organization_uuid = $1 AND active = true
                 ORDER BY event_name, created_at"
            )
            .bind(organization_uuid)
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .filter_map(|row| {
                    let headers: Option<JsonValue> = row.try_get("headers").ok().flatten();
                    
                    Some(Webhook {
                        id: row.get("id"),
                        organization_uuid: row.get("organization_uuid"),
                        event_name: row.get("event_name"),
                        url: row.get("url"),
                        secret: row.try_get("secret").ok().flatten(),
                        headers,
                        active: row.get("active"),
                        created_by: row.get("created_by"),
                        created_at: row.get("created_at"),
                        updated_at: row.get("updated_at"),
                    })
                })
                .collect())
        }
        DatabasePool::Sqlite(p) => {
            let rows = sqlx::query(
                "SELECT id, organization_uuid, event_name, url, secret, headers, active, created_by, created_at, updated_at
                 FROM event_webhooks 
                 WHERE organization_uuid = ?1 AND active = 1
                 ORDER BY event_name, created_at"
            )
            .bind(organization_uuid)
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .filter_map(|row| {
                    let headers_str: Option<String> = row.try_get("headers").ok().flatten();
                    let headers = headers_str.and_then(|s| serde_json::from_str(&s).ok());
                    
                    Some(Webhook {
                        id: row.get("id"),
                        organization_uuid: row.get("organization_uuid"),
                        event_name: row.get("event_name"),
                        url: row.get("url"),
                        secret: row.try_get("secret").ok().flatten(),
                        headers,
                        active: row.get("active"),
                        created_by: row.get("created_by"),
                        created_at: row.get("created_at"),
                        updated_at: row.get("updated_at"),
                    })
                })
                .collect())
        }
    }
}

/// Create a new webhook in the database
pub async fn create_webhook(
    pool: &DatabasePool,
    organization_uuid: &str,
    created_by: &str,
    request: &CreateWebhookRequest,
) -> Result<String, sqlx::Error> {
    let webhook_id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now();

    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query(
                "INSERT INTO event_webhooks 
                 (id, organization_uuid, event_name, url, secret, headers, active, created_by, created_at, updated_at) 
                 VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?, ?)"
            )
            .bind(&webhook_id)
            .bind(organization_uuid)
            .bind(&request.event_name)
            .bind(&request.url)
            .bind(&request.secret)
            .bind(&request.headers)
            .bind(created_by)
            .bind(now)
            .bind(now)
            .execute(p)
            .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO event_webhooks 
                 (id, organization_uuid, event_name, url, secret, headers, active, created_by, created_at, updated_at) 
                 VALUES ($1, $2, $3, $4, $5, $6, true, $7, $8, $9)"
            )
            .bind(&webhook_id)
            .bind(organization_uuid)
            .bind(&request.event_name)
            .bind(&request.url)
            .bind(&request.secret)
            .bind(&request.headers)
            .bind(created_by)
            .bind(now)
            .bind(now)
            .execute(p)
            .await?;
        }
        DatabasePool::Sqlite(p) => {
            let headers_json = request.headers.as_ref()
                .map(|h| serde_json::to_string(h).unwrap_or_default());
            
            sqlx::query(
                "INSERT INTO event_webhooks 
                 (id, organization_uuid, event_name, url, secret, headers, active, created_by, created_at, updated_at) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8, ?9)"
            )
            .bind(&webhook_id)
            .bind(organization_uuid)
            .bind(&request.event_name)
            .bind(&request.url)
            .bind(&request.secret)
            .bind(&headers_json)
            .bind(created_by)
            .bind(now)
            .bind(now)
            .execute(p)
            .await?;
        }
    }

    Ok(webhook_id)
}

/// Update a webhook in the database
pub async fn update_webhook(
    pool: &DatabasePool,
    webhook_id: &str,
    organization_uuid: &str,
    request: &UpdateWebhookRequest,
) -> Result<(), sqlx::Error> {
    let now = Utc::now();

    // Build dynamic update query
    let mut updates = Vec::new();
    let mut bind_index = 1;

    if request.event_name.is_some() {
        updates.push(format!("event_name = ${}", bind_index));
        bind_index += 1;
    }
    if request.url.is_some() {
        updates.push(format!("url = ${}", bind_index));
        bind_index += 1;
    }
    if request.secret.is_some() {
        updates.push(format!("secret = ${}", bind_index));
        bind_index += 1;
    }
    if request.headers.is_some() {
        updates.push(format!("headers = ${}", bind_index));
        bind_index += 1;
    }
    if request.active.is_some() {
        updates.push(format!("active = ${}", bind_index));
        bind_index += 1;
    }
    updates.push(format!("updated_at = ${}", bind_index));
    bind_index += 1;

    if updates.is_empty() {
        return Ok(()); // Nothing to update
    }

    let update_clause = updates.join(", ");

    match pool {
        DatabasePool::MySql(p) => {
            // MySQL uses ? placeholders (not ?1, ?2, etc.)
            // Replace $1, $2, etc. with just ?
            // Replace in reverse order to avoid $1 matching in $10, $11, etc.
            let mut mysql_update_clause = update_clause.clone();
            for i in (1..=bind_index).rev() {
                mysql_update_clause = mysql_update_clause.replace(&format!("${}", i), "?");
            }
            
            let query = format!(
                "UPDATE event_webhooks SET {} WHERE id = ? AND organization_uuid = ?",
                mysql_update_clause
            );
            
            let mut query_builder = sqlx::query(&query);
            
            if let Some(event_name) = &request.event_name {
                query_builder = query_builder.bind(event_name);
            }
            if let Some(url) = &request.url {
                query_builder = query_builder.bind(url);
            }
            if let Some(secret) = &request.secret {
                query_builder = query_builder.bind(secret);
            }
            if let Some(headers) = &request.headers {
                query_builder = query_builder.bind(headers);
            }
            if let Some(active) = request.active {
                query_builder = query_builder.bind(active);
            }
            query_builder = query_builder.bind(now);
            query_builder = query_builder.bind(webhook_id);
            query_builder = query_builder.bind(organization_uuid);
            
            query_builder.execute(p).await?;
        }
        DatabasePool::Postgres(p) => {
            let query = format!(
                "UPDATE event_webhooks SET {} WHERE id = ${} AND organization_uuid = ${}",
                update_clause, bind_index, bind_index + 1
            );
            
            let mut query_builder = sqlx::query(&query);
            
            if let Some(event_name) = &request.event_name {
                query_builder = query_builder.bind(event_name);
            }
            if let Some(url) = &request.url {
                query_builder = query_builder.bind(url);
            }
            if let Some(secret) = &request.secret {
                query_builder = query_builder.bind(secret);
            }
            if let Some(headers) = &request.headers {
                query_builder = query_builder.bind(headers);
            }
            if let Some(active) = request.active {
                query_builder = query_builder.bind(active);
            }
            query_builder = query_builder.bind(now);
            query_builder = query_builder.bind(webhook_id);
            query_builder = query_builder.bind(organization_uuid);
            
            query_builder.execute(p).await?;
        }
        DatabasePool::Sqlite(p) => {
            // SQLite uses ?1, ?2, etc.
            let query = format!(
                "UPDATE event_webhooks SET {} WHERE id = ?{} AND organization_uuid = ?{}",
                update_clause.replace('$', "?"), bind_index, bind_index + 1
            );
            
            let mut query_builder = sqlx::query(&query);
            
            if let Some(event_name) = &request.event_name {
                query_builder = query_builder.bind(event_name);
            }
            if let Some(url) = &request.url {
                query_builder = query_builder.bind(url);
            }
            if let Some(secret) = &request.secret {
                query_builder = query_builder.bind(secret);
            }
            if let Some(headers) = &request.headers {
                let headers_json = serde_json::to_string(headers).unwrap_or_default();
                query_builder = query_builder.bind(headers_json);
            }
            if let Some(active) = request.active {
                query_builder = query_builder.bind(active);
            }
            query_builder = query_builder.bind(now);
            query_builder = query_builder.bind(webhook_id);
            query_builder = query_builder.bind(organization_uuid);
            
            query_builder.execute(p).await?;
        }
    }

    Ok(())
}

/// Delete a webhook from the database
pub async fn delete_webhook(
    pool: &DatabasePool,
    webhook_id: &str,
    organization_uuid: &str,
) -> Result<(), sqlx::Error> {
    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query("DELETE FROM event_webhooks WHERE id = ? AND organization_uuid = ?")
                .bind(webhook_id)
                .bind(organization_uuid)
                .execute(p)
                .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query("DELETE FROM event_webhooks WHERE id = $1 AND organization_uuid = $2")
                .bind(webhook_id)
                .bind(organization_uuid)
                .execute(p)
                .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query("DELETE FROM event_webhooks WHERE id = ?1 AND organization_uuid = ?2")
                .bind(webhook_id)
                .bind(organization_uuid)
                .execute(p)
                .await?;
        }
    }

    Ok(())
}

/// Get a webhook by ID (for verification)
pub async fn get_webhook(
    pool: &DatabasePool,
    webhook_id: &str,
    organization_uuid: &str,
) -> Result<Option<Webhook>, sqlx::Error> {
    match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query(
                "SELECT id, organization_uuid, event_name, url, secret, headers, active, created_by, created_at, updated_at
                 FROM event_webhooks 
                 WHERE id = ? AND organization_uuid = ?"
            )
            .bind(webhook_id)
            .bind(organization_uuid)
            .fetch_optional(p)
            .await?;

            Ok(row.and_then(|row| {
                let headers: Option<JsonValue> = row.try_get("headers").ok().flatten();
                
                Some(Webhook {
                    id: row.get("id"),
                    organization_uuid: row.get("organization_uuid"),
                    event_name: row.get("event_name"),
                    url: row.get("url"),
                    secret: row.try_get("secret").ok().flatten(),
                    headers,
                    active: row.get("active"),
                    created_by: row.get("created_by"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                })
            }))
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT id, organization_uuid, event_name, url, secret, headers, active, created_by, created_at, updated_at
                 FROM event_webhooks 
                 WHERE id = $1 AND organization_uuid = $2"
            )
            .bind(webhook_id)
            .bind(organization_uuid)
            .fetch_optional(p)
            .await?;

            Ok(row.and_then(|row| {
                let headers: Option<JsonValue> = row.try_get("headers").ok().flatten();
                
                Some(Webhook {
                    id: row.get("id"),
                    organization_uuid: row.get("organization_uuid"),
                    event_name: row.get("event_name"),
                    url: row.get("url"),
                    secret: row.try_get("secret").ok().flatten(),
                    headers,
                    active: row.get("active"),
                    created_by: row.get("created_by"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                })
            }))
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT id, organization_uuid, event_name, url, secret, headers, active, created_by, created_at, updated_at
                 FROM event_webhooks 
                 WHERE id = ?1 AND organization_uuid = ?2"
            )
            .bind(webhook_id)
            .bind(organization_uuid)
            .fetch_optional(p)
            .await?;

            Ok(row.and_then(|row| {
                let headers_str: Option<String> = row.try_get("headers").ok().flatten();
                let headers = headers_str.and_then(|s| serde_json::from_str(&s).ok());
                
                Some(Webhook {
                    id: row.get("id"),
                    organization_uuid: row.get("organization_uuid"),
                    event_name: row.get("event_name"),
                    url: row.get("url"),
                    secret: row.try_get("secret").ok().flatten(),
                    headers,
                    active: row.get("active"),
                    created_by: row.get("created_by"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                })
            }))
        }
    }
}

/// Send a webhook HTTP POST request
///
/// This function sends the event data to the webhook URL with optional
/// HMAC signature for verification.
pub async fn send_webhook(webhook: &Webhook, event: &Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    // Build webhook payload
    let payload = serde_json::json!({
        "event": {
            "name": event.name,
            "timestamp": event.timestamp.to_rfc3339(),
            "organization_uuid": event.organization_uuid,
            "user_uuid": event.user_uuid,
            "payload": event.payload.data
        },
        "webhook_id": webhook.id
    });

    let payload_json = serde_json::to_string(&payload)?;

    // Build request
    let mut request = client
        .post(&webhook.url)
        .header("Content-Type", "application/json")
        .header("User-Agent", "Flextide-Webhook/1.0")
        .body(payload_json.clone());

    // Add custom headers if provided
    if let Some(headers) = &webhook.headers {
        if let Some(headers_map) = headers.as_object() {
            for (key, value) in headers_map {
                if let Some(header_value) = value.as_str() {
                    request = request.header(key, header_value);
                }
            }
        }
    }

    // Add HMAC signature if secret is provided
    if let Some(secret) = &webhook.secret {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|e| format!("Failed to create HMAC: {}", e))?;
        mac.update(payload_json.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());
        request = request.header("X-Webhook-Signature", format!("sha256={}", signature));
    }

    // Send request
    debug!("Sending webhook to {} for event {}", webhook.url, event.name);
    
    let response = request.send().await?;
    let status = response.status();

    if status.is_success() {
        debug!("Webhook delivered successfully: {} (status: {})", webhook.url, status);
        Ok(())
    } else {
        let error_text = response.text().await.unwrap_or_default();
        warn!(
            "Webhook delivery failed: {} (status: {}, error: {})",
            webhook.url, status, error_text
        );
        Err(format!("Webhook delivery failed with status {}: {}", status, error_text).into())
    }
}

