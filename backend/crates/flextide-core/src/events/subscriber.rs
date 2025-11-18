//! Event Subscriber
//!
//! Defines the interface for event subscribers and subscriber types.

use crate::events::types::{Event, EventPayload};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Trait for event subscribers that can handle events
#[async_trait]
pub trait EventSubscriber: Send + Sync {
    /// Handle an event
    ///
    /// This method is called when an event matching the subscriber's
    /// event name is emitted.
    async fn handle_event(&self, event: &Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Get the event name this subscriber listens to
    fn event_name(&self) -> &str;

    /// Get a unique identifier for this subscriber
    fn subscriber_id(&self) -> &str;
}

/// Type of event subscriber
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventSubscriberType {
    /// Database-backed subscriber (loaded from database)
    Database {
        /// Subscriber ID from database
        id: String,
        /// Type identifier (e.g., "webhook", "function", "class_method")
        subscriber_type: String,
        /// Configuration as JSON
        config: JsonValue,
    },
    /// Runtime-registered subscriber
    Runtime {
        /// Unique identifier for the runtime subscriber
        id: String,
    },
}

/// Database-backed event subscription record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseEventSubscription {
    /// Subscription ID
    pub id: String,
    /// Event name to subscribe to
    pub event_name: String,
    /// Subscriber type (e.g., "webhook", "function", "class_method")
    pub subscriber_type: String,
    /// Subscriber configuration as JSON
    pub config: JsonValue,
    /// Whether the subscription is active
    pub active: bool,
    /// Optional organization UUID for organization-scoped subscriptions
    pub organization_uuid: Option<String>,
    /// Source that created this subscription (e.g., "plugin_name", "system")
    pub created_from: String,
}

/// Runtime event subscription
pub struct RuntimeEventSubscription {
    /// Unique identifier
    pub id: String,
    /// Event name to subscribe to
    pub event_name: String,
    /// The subscriber implementation
    pub subscriber: Box<dyn EventSubscriber>,
}

