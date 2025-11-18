# Event System

The Flextide event system provides a flexible, extensible way to handle events throughout the platform. It supports both database-backed subscriptions (cached in memory) and runtime-registered subscribers.

## Features

- **Event Emission**: Any part of Flextide can emit events with JSON payloads
- **Database-Backed Subscriptions**: Store event subscriptions in the database, loaded into memory at startup
- **Runtime Subscriptions**: Register event handlers programmatically at runtime
- **Organization Scoping**: Support for organization-scoped event subscriptions
- **Extensible Architecture**: Designed to support future connectors (webhooks, Kafka, etc.)

## Architecture

The event system consists of:

- **Event**: A named event with a JSON payload
- **EventDispatcher**: Manages subscribers and dispatches events
- **EventSubscriber**: Trait for event handlers
- **Database Subscriptions**: Persistent subscriptions stored in the database
- **Runtime Subscriptions**: In-memory subscriptions registered at runtime

## Usage

### Initialization

Initialize the event system at application startup:

```rust
use flextide_core::events::{EventDispatcher, initialize};
use flextide_core::database::DatabasePool;

let dispatcher = EventDispatcher::new();
initialize(&dispatcher, &pool).await?;
```

### Emitting Events

Emit events from anywhere in the application:

```rust
use flextide_core::events::{Event, EventPayload};
use serde_json::json;

// Create an event
let event = Event::new(
    "project.created",
    EventPayload::new(json!({
        "project_id": "123",
        "project_name": "My Project"
    }))
)
.with_organization("org-uuid")
.with_user("user-uuid");

// Emit the event
dispatcher.emit(event).await;
```

### Creating Runtime Subscribers

Implement the `EventSubscriber` trait to create custom event handlers:

```rust
use flextide_core::events::{Event, EventSubscriber};
use async_trait::async_trait;

struct MyEventHandler {
    event_name: String,
    subscriber_id: String,
}

#[async_trait]
impl EventSubscriber for MyEventHandler {
    async fn handle_event(&self, event: &Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Received event: {} with payload: {:?}", event.name, event.payload.data);
        // Handle the event
        Ok(())
    }

    fn event_name(&self) -> &str {
        &self.event_name
    }

    fn subscriber_id(&self) -> &str {
        &self.subscriber_id
    }
}

// Register the subscriber
let handler = Box::new(MyEventHandler {
    event_name: "project.created".to_string(),
    subscriber_id: "my-handler-1".to_string(),
});
dispatcher.subscribe(handler);
```

### Database-Backed Subscriptions

Database subscriptions are automatically loaded at startup. To create a subscription in the database:

```rust
use flextide_core::events::database::create_event_subscription;
use flextide_core::events::subscriber::DatabaseEventSubscription;
use serde_json::json;
use uuid::Uuid;

let subscription = DatabaseEventSubscription {
    id: Uuid::new_v4().to_string(),
    event_name: "project.created".to_string(),
    subscriber_type: "webhook".to_string(),
    config: json!({
        "url": "https://example.com/webhook",
        "method": "POST"
    }),
    active: true,
    organization_uuid: Some("org-uuid".to_string()),
    created_from: "system".to_string(),
};

create_event_subscription(&pool, &subscription).await?;

// Reload subscriptions to include the new one
dispatcher.load_database_subscriptions(&pool).await?;
```

## Event Naming Convention

Use dot-separated names for events:
- `project.created`
- `project.updated`
- `project.deleted`
- `user.registered`
- `workflow.executed`

## Organization Scoping

Events can be scoped to organizations. When emitting an event with an organization UUID, only subscriptions that match that organization (or have no organization UUID) will receive the event.

```rust
let event = Event::new("project.created", payload)
    .with_organization("org-uuid");
```

## Future Connectors

The event system is designed to support connectors for:
- **Webhooks**: HTTP POST requests to external URLs
- **Kafka**: Publishing events to Kafka topics
- **Function Calls**: Calling internal functions
- **Custom Connectors**: Extend with your own connector types

Connectors are implemented in the `handle_database_subscription` function in `dispatcher.rs`.

## Database Schema

The `event_subscriptions` table stores persistent subscriptions:

- `id`: Unique identifier (UUID)
- `event_name`: Event name to subscribe to
- `subscriber_type`: Type of subscriber (webhook, kafka, etc.)
- `config`: JSON configuration for the subscriber
- `active`: Whether the subscription is active
- `organization_uuid`: Optional organization UUID
- `created_from`: Source that created the subscription

## Performance Considerations

- Database subscriptions are loaded once at startup and cached in memory
- Event dispatching is asynchronous and non-blocking
- Subscriber errors are logged but don't stop event processing
- Use `DashMap` for thread-safe concurrent access to subscriptions

## Error Handling

Event subscribers should handle errors gracefully. Errors in subscribers are logged but don't prevent other subscribers from receiving the event.

```rust
async fn handle_event(&self, event: &Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Handle the event
    // Return Ok(()) on success, Err(...) on failure
    // Errors will be logged but won't stop other subscribers
    Ok(())
}
```

