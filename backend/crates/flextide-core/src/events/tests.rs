//! Tests for the event system

use crate::events::{
    Event, EventDispatcher, EventPayload, EventSubscriber,
};
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Test subscriber that collects received events
#[derive(Clone)]
struct TestSubscriber {
    event_name: String,
    subscriber_id: String,
    received_events: Arc<Mutex<Vec<Event>>>,
}

impl TestSubscriber {
    fn new(event_name: impl Into<String>, subscriber_id: impl Into<String>) -> Self {
        Self {
            event_name: event_name.into(),
            subscriber_id: subscriber_id.into(),
            received_events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn get_received_events(&self) -> Vec<Event> {
        self.received_events.lock().await.clone()
    }
}

#[async_trait]
impl EventSubscriber for TestSubscriber {
    async fn handle_event(&self, event: &Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.received_events.lock().await.push(event.clone());
        Ok(())
    }

    fn event_name(&self) -> &str {
        &self.event_name
    }

    fn subscriber_id(&self) -> &str {
        &self.subscriber_id
    }
}

/// Test subscriber that returns an error
struct ErrorSubscriber {
    event_name: String,
    subscriber_id: String,
}

impl ErrorSubscriber {
    fn new(event_name: impl Into<String>, subscriber_id: impl Into<String>) -> Self {
        Self {
            event_name: event_name.into(),
            subscriber_id: subscriber_id.into(),
        }
    }
}

#[async_trait]
impl EventSubscriber for ErrorSubscriber {
    async fn handle_event(&self, _event: &Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Err("Test error".into())
    }

    fn event_name(&self) -> &str {
        &self.event_name
    }

    fn subscriber_id(&self) -> &str {
        &self.subscriber_id
    }
}

#[tokio::test]
async fn test_event_creation() {
    let payload = EventPayload::new(json!({"key": "value"}));
    let event = Event::new("test.event", payload);

    assert_eq!(event.name, "test.event");
    assert_eq!(event.payload.data["key"], "value");
    assert!(event.organization_uuid.is_none());
    assert!(event.user_uuid.is_none());
}

#[tokio::test]
async fn test_event_with_organization() {
    let payload = EventPayload::new(json!({}));
    let event = Event::new("test.event", payload)
        .with_organization("org-123");

    assert_eq!(event.organization_uuid, Some("org-123".to_string()));
}

#[tokio::test]
async fn test_event_with_user() {
    let payload = EventPayload::new(json!({}));
    let event = Event::new("test.event", payload)
        .with_user("user-456");

    assert_eq!(event.user_uuid, Some("user-456".to_string()));
}

#[tokio::test]
async fn test_event_payload_from_serializable() {
    #[derive(serde::Serialize)]
    struct TestData {
        name: String,
        count: i32,
    }

    let data = TestData {
        name: "test".to_string(),
        count: 42,
    };

    let payload = EventPayload::from_serializable(&data).unwrap();
    assert_eq!(payload.data["name"], "test");
    assert_eq!(payload.data["count"], 42);
}

#[tokio::test]
async fn test_event_payload_empty() {
    let payload = EventPayload::empty();
    assert!(payload.data.is_object());
    assert!(payload.data.as_object().unwrap().is_empty());
}

#[tokio::test]
async fn test_runtime_subscriber() {
    let dispatcher = EventDispatcher::new();
    let subscriber = TestSubscriber::new("test.event", "subscriber-1");

    dispatcher.subscribe(Box::new(subscriber.clone()));

    let event = Event::new("test.event", EventPayload::new(json!({"data": "value"})));
    dispatcher.emit(event.clone()).await;

    let received = subscriber.get_received_events().await;
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].name, "test.event");
}

#[tokio::test]
async fn test_multiple_subscribers_same_event() {
    let dispatcher = EventDispatcher::new();
    let subscriber1 = TestSubscriber::new("test.event", "subscriber-1");
    let subscriber2 = TestSubscriber::new("test.event", "subscriber-2");

    dispatcher.subscribe(Box::new(subscriber1.clone()));
    dispatcher.subscribe(Box::new(subscriber2.clone()));

    let event = Event::new("test.event", EventPayload::new(json!({})));
    dispatcher.emit(event).await;

    let received1 = subscriber1.get_received_events().await;
    let received2 = subscriber2.get_received_events().await;

    assert_eq!(received1.len(), 1);
    assert_eq!(received2.len(), 1);
}

#[tokio::test]
async fn test_subscriber_different_events() {
    let dispatcher = EventDispatcher::new();
    let subscriber1 = TestSubscriber::new("event.1", "subscriber-1");
    let subscriber2 = TestSubscriber::new("event.2", "subscriber-2");

    dispatcher.subscribe(Box::new(subscriber1.clone()));
    dispatcher.subscribe(Box::new(subscriber2.clone()));

    let event1 = Event::new("event.1", EventPayload::new(json!({})));
    dispatcher.emit(event1).await;

    let received1 = subscriber1.get_received_events().await;
    let received2 = subscriber2.get_received_events().await;

    assert_eq!(received1.len(), 1);
    assert_eq!(received2.len(), 0); // Should not receive event.1
}

#[tokio::test]
async fn test_subscriber_error_handling() {
    let dispatcher = EventDispatcher::new();
    let error_subscriber = ErrorSubscriber::new("test.event", "error-subscriber");
    let normal_subscriber = TestSubscriber::new("test.event", "normal-subscriber");

    dispatcher.subscribe(Box::new(error_subscriber));
    dispatcher.subscribe(Box::new(normal_subscriber.clone()));

    // Error in one subscriber should not prevent others from receiving the event
    let event = Event::new("test.event", EventPayload::new(json!({})));
    dispatcher.emit(event).await;

    let received = normal_subscriber.get_received_events().await;
    assert_eq!(received.len(), 1); // Normal subscriber should still receive the event
}

#[tokio::test]
async fn test_unsubscribe() {
    let dispatcher = EventDispatcher::new();
    let subscriber = TestSubscriber::new("test.event", "subscriber-1");

    dispatcher.subscribe(Box::new(subscriber.clone()));

    // Emit first event
    let event1 = Event::new("test.event", EventPayload::new(json!({})));
    dispatcher.emit(event1).await;

    // Unsubscribe
    let removed = dispatcher.unsubscribe("test.event", "subscriber-1");
    assert!(removed);

    // Emit second event
    let event2 = Event::new("test.event", EventPayload::new(json!({})));
    dispatcher.emit(event2).await;

    let received = subscriber.get_received_events().await;
    assert_eq!(received.len(), 1); // Should only have the first event
}

#[tokio::test]
async fn test_unsubscribe_nonexistent() {
    let dispatcher = EventDispatcher::new();
    let removed = dispatcher.unsubscribe("nonexistent.event", "subscriber-1");
    assert!(!removed);
}

#[tokio::test]
async fn test_subscriber_count() {
    let dispatcher = EventDispatcher::new();
    
    assert_eq!(dispatcher.subscriber_count("test.event"), 0);

    dispatcher.subscribe(Box::new(TestSubscriber::new("test.event", "subscriber-1")));
    assert_eq!(dispatcher.subscriber_count("test.event"), 1);

    dispatcher.subscribe(Box::new(TestSubscriber::new("test.event", "subscriber-2")));
    assert_eq!(dispatcher.subscriber_count("test.event"), 2);

    dispatcher.subscribe(Box::new(TestSubscriber::new("other.event", "subscriber-3")));
    assert_eq!(dispatcher.subscriber_count("test.event"), 2); // Should still be 2
    assert_eq!(dispatcher.subscriber_count("other.event"), 1);
}

#[tokio::test]
async fn test_event_payload_from_json_value() {
    let json_value = json!({"key": "value", "number": 42});
    let payload: EventPayload = json_value.clone().into();

    assert_eq!(payload.data, json_value);
}

#[tokio::test]
async fn test_event_payload_from_map() {
    use serde_json::Map;
    
    let mut map = Map::new();
    map.insert("key".to_string(), json!("value"));
    let payload: EventPayload = map.into();

    assert_eq!(payload.data["key"], "value");
}

#[tokio::test]
async fn test_multiple_events_same_subscriber() {
    let dispatcher = EventDispatcher::new();
    let subscriber = TestSubscriber::new("test.event", "subscriber-1");

    dispatcher.subscribe(Box::new(subscriber.clone()));

    // Emit multiple events
    for i in 0..5 {
        let event = Event::new("test.event", EventPayload::new(json!({"index": i})));
        dispatcher.emit(event).await;
    }

    let received = subscriber.get_received_events().await;
    assert_eq!(received.len(), 5);
    
    for (i, event) in received.iter().enumerate() {
        assert_eq!(event.payload.data["index"], i);
    }
}

#[tokio::test]
async fn test_event_timestamp() {
    let before = chrono::Utc::now();
    let payload = EventPayload::new(json!({}));
    let event = Event::new("test.event", payload);
    let after = chrono::Utc::now();

    assert!(event.timestamp >= before);
    assert!(event.timestamp <= after);
}

