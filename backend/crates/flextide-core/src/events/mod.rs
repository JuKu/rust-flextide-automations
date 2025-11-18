//! Event System
//!
//! Provides a flexible event system for the Flextide platform that supports:
//! - Event emission with JSON payloads
//! - Database-backed event subscriptions (cached in memory)
//! - Runtime event subscriptions
//! - Extensible architecture for future connectors (webhooks, Kafka, etc.)

mod database;
mod dispatcher;
mod subscriber;
mod types;
mod webhooks;

#[cfg(test)]
mod tests;

pub use dispatcher::{EventDispatcher, EventDispatcherError};
pub use subscriber::{EventSubscriber, EventSubscriberType};
pub use types::{Event, EventPayload};
pub use webhooks::{
    CreateWebhookRequest, UpdateWebhookRequest, Webhook,
    create_webhook, delete_webhook, get_webhook, load_webhooks, load_webhooks_by_organization,
    send_webhook, update_webhook,
};

/// Initialize the event system by loading database-backed subscriptions and webhooks
///
/// This should be called once at application startup to load all
/// event subscriptions and webhooks from the database into memory.
pub async fn initialize(
    dispatcher: &EventDispatcher,
    pool: &crate::database::DatabasePool,
) -> Result<(), EventDispatcherError> {
    dispatcher.load_database_subscriptions(pool).await?;
    dispatcher.load_webhooks(pool).await?;
    Ok(())
}

