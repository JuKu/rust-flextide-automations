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

#[cfg(test)]
mod tests;

pub use dispatcher::{EventDispatcher, EventDispatcherError};
pub use subscriber::{EventSubscriber, EventSubscriberType};
pub use types::{Event, EventPayload};

/// Initialize the event system by loading database-backed subscriptions
///
/// This should be called once at application startup to load all
/// event subscriptions from the database into memory.
pub async fn initialize(
    dispatcher: &EventDispatcher,
    pool: &crate::database::DatabasePool,
) -> Result<(), EventDispatcherError> {
    dispatcher.load_database_subscriptions(pool).await
}

