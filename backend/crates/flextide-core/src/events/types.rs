//! Event Types
//!
//! Core types for the event system.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// An event that can be emitted and handled by subscribers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Event name/identifier (e.g., "project.created", "user.deleted")
    pub name: String,
    /// Event payload as JSON
    pub payload: EventPayload,
    /// Timestamp when the event was emitted
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Optional organization UUID for organization-scoped events
    pub organization_uuid: Option<String>,
    /// Optional user UUID who triggered the event
    pub user_uuid: Option<String>,
}

impl Event {
    /// Create a new event
    pub fn new(name: impl Into<String>, payload: EventPayload) -> Self {
        Self {
            name: name.into(),
            payload,
            timestamp: chrono::Utc::now(),
            organization_uuid: None,
            user_uuid: None,
        }
    }

    /// Create a new event with organization context
    pub fn with_organization(mut self, organization_uuid: impl Into<String>) -> Self {
        self.organization_uuid = Some(organization_uuid.into());
        self
    }

    /// Create a new event with user context
    pub fn with_user(mut self, user_uuid: impl Into<String>) -> Self {
        self.user_uuid = Some(user_uuid.into());
        self
    }
}

/// Event payload - a wrapper around JSON value for type safety
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPayload {
    /// The actual JSON payload
    pub data: JsonValue,
}

impl EventPayload {
    /// Create a new event payload from a JSON value
    pub fn new(data: JsonValue) -> Self {
        Self { data }
    }

    /// Create an empty event payload
    pub fn empty() -> Self {
        Self {
            data: JsonValue::Object(serde_json::Map::new()),
        }
    }

    /// Create an event payload from a serializable type
    pub fn from_serializable<T: Serialize>(value: &T) -> Result<Self, serde_json::Error> {
        Ok(Self {
            data: serde_json::to_value(value)?,
        })
    }
}

impl From<JsonValue> for EventPayload {
    fn from(value: JsonValue) -> Self {
        Self::new(value)
    }
}

impl From<serde_json::Map<String, JsonValue>> for EventPayload {
    fn from(map: serde_json::Map<String, JsonValue>) -> Self {
        Self {
            data: JsonValue::Object(map),
        }
    }
}

