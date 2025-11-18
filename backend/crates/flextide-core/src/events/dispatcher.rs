//! Event Dispatcher
//!
//! Manages event subscriptions and dispatches events to subscribers.

use crate::database::DatabasePool;
use crate::events::database::load_event_subscriptions;
use crate::events::subscriber::{DatabaseEventSubscription, EventSubscriber};
use crate::events::types::Event;
use dashmap::DashMap;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error, warn};

/// Event dispatcher that manages subscribers and dispatches events
#[derive(Clone)]
pub struct EventDispatcher {
    /// Database-backed subscriptions (cached in memory)
    database_subscriptions: Arc<DashMap<String, Vec<DatabaseEventSubscription>>>,
    /// Runtime-registered subscriptions
    runtime_subscriptions: Arc<DashMap<String, Vec<Arc<dyn EventSubscriber>>>>,
}

impl EventDispatcher {
    /// Create a new event dispatcher
    pub fn new() -> Self {
        Self {
            database_subscriptions: Arc::new(DashMap::new()),
            runtime_subscriptions: Arc::new(DashMap::new()),
        }
    }

    /// Load all active event subscriptions from the database into memory
    ///
    /// This should be called once at application startup to cache
    /// all database-backed subscriptions.
    pub async fn load_database_subscriptions(
        &self,
        pool: &DatabasePool,
    ) -> Result<(), EventDispatcherError> {
        debug!("Loading event subscriptions from database");

        let subscriptions = load_event_subscriptions(pool).await?;
        let total_count = subscriptions.len();

        // Clear existing subscriptions
        self.database_subscriptions.clear();

        // Group subscriptions by event name
        for subscription in subscriptions {
            if !subscription.active {
                continue;
            }

            self.database_subscriptions
                .entry(subscription.event_name.clone())
                .or_insert_with(Vec::new)
                .push(subscription);
        }

        debug!(
            "Loaded {} event subscriptions for {} events",
            total_count,
            self.database_subscriptions.len()
        );

        Ok(())
    }

    /// Register a runtime event subscriber
    ///
    /// Runtime subscribers are registered in memory and persist until
    /// the application restarts or they are explicitly removed.
    pub fn subscribe(&self, subscriber: Box<dyn EventSubscriber>) {
        let event_name = subscriber.event_name().to_string();
        let subscriber_id = subscriber.subscriber_id().to_string();

        debug!(
            "Registering runtime subscriber: id={}, event={}",
            subscriber_id, event_name
        );

        self.runtime_subscriptions
            .entry(event_name)
            .or_insert_with(Vec::new)
            .push(Arc::from(subscriber));
    }

    /// Unregister a runtime event subscriber by ID
    pub fn unsubscribe(&self, event_name: &str, subscriber_id: &str) -> bool {
        if let Some(mut subscribers) = self.runtime_subscriptions.get_mut(event_name) {
            let initial_len = subscribers.len();
            subscribers.retain(|s| s.subscriber_id() != subscriber_id);
            let removed = initial_len != subscribers.len();

            if removed {
                debug!(
                    "Unregistered runtime subscriber: id={}, event={}",
                    subscriber_id, event_name
                );
            }

            return removed;
        }
        false
    }

    /// Emit an event to all registered subscribers
    ///
    /// This method:
    /// 1. Finds all subscribers (database-backed and runtime) for the event
    /// 2. Calls each subscriber's handle_event method
    /// 3. Logs errors but continues processing other subscribers
    pub async fn emit(&self, event: Event) {
        let event_name = &event.name;
        debug!("Emitting event: {}", event_name);

        // Handle database-backed subscriptions
        if let Some(subscriptions) = self.database_subscriptions.get(event_name) {
            for subscription in subscriptions.iter() {
                // Check organization scope if applicable
                if let Some(ref org_uuid) = subscription.organization_uuid {
                    if event.organization_uuid.as_ref() != Some(org_uuid) {
                        continue;
                    }
                }

                // Handle database subscription
                // Note: This is where connectors (webhooks, Kafka, etc.) would be invoked
                // For now, we just log - connectors will be implemented later
                debug!(
                    "Processing database subscription: id={}, type={}",
                    subscription.id, subscription.subscriber_type
                );

                // TODO: Implement connector system here
                // This will dispatch to webhook connectors, Kafka connectors, etc.
                match handle_database_subscription(subscription, &event).await {
                    Ok(_) => {
                        debug!("Successfully processed database subscription: {}", subscription.id);
                    }
                    Err(e) => {
                        error!(
                            "Error processing database subscription {}: {}",
                            subscription.id, e
                        );
                    }
                }
            }
        }

        // Handle runtime subscriptions
        if let Some(subscribers) = self.runtime_subscriptions.get(event_name) {
            for subscriber in subscribers.iter() {
                match subscriber.handle_event(&event).await {
                    Ok(_) => {
                        debug!(
                            "Successfully processed runtime subscriber: {}",
                            subscriber.subscriber_id()
                        );
                    }
                    Err(e) => {
                        error!(
                            "Error processing runtime subscriber {}: {}",
                            subscriber.subscriber_id(),
                            e
                        );
                    }
                }
            }
        }
    }

    /// Get the number of subscribers for an event
    pub fn subscriber_count(&self, event_name: &str) -> usize {
        let db_count = self
            .database_subscriptions
            .get(event_name)
            .map(|v| v.len())
            .unwrap_or(0);
        let runtime_count = self
            .runtime_subscriptions
            .get(event_name)
            .map(|v| v.len())
            .unwrap_or(0);
        db_count + runtime_count
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle a database-backed subscription
///
/// This function will be extended to support different connector types
/// (webhooks, Kafka, etc.) in the future.
async fn handle_database_subscription(
    subscription: &DatabaseEventSubscription,
    _event: &Event,
) -> Result<(), EventDispatcherError> {
    match subscription.subscriber_type.as_str() {
        "webhook" => {
            // TODO: Implement webhook connector
            warn!("Webhook connector not yet implemented for subscription: {}", subscription.id);
            Ok(())
        }
        "kafka" => {
            // TODO: Implement Kafka connector
            warn!("Kafka connector not yet implemented for subscription: {}", subscription.id);
            Ok(())
        }
        "function" => {
            // TODO: Implement function call connector
            warn!("Function connector not yet implemented for subscription: {}", subscription.id);
            Ok(())
        }
        _ => {
            warn!(
                "Unknown subscriber type '{}' for subscription: {}",
                subscription.subscriber_type, subscription.id
            );
            Ok(())
        }
    }
}

/// Event dispatcher errors
#[derive(Debug, Error)]
pub enum EventDispatcherError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Failed to load event subscriptions: {0}")]
    LoadError(String),

    #[error("Invalid subscription configuration: {0}")]
    InvalidConfig(String),
}

