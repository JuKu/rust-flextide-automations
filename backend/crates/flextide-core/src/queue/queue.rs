use serde_json::Value;
use std::future::Future;
use std::time::Duration;
use thiserror::Error;

/// A message retrieved from the queue
#[derive(Debug, Clone)]
pub struct QueueMessage {
    /// Unique identifier for this message (provider-specific)
    pub id: String,
    /// The JSON payload
    pub payload: Value,
    /// Receipt handle or token for acknowledging/deleting the message
    /// (provider-specific, e.g., SQS receipt handle)
    pub receipt_handle: Option<String>,
}

/// Errors that can occur during queue operations
#[derive(Debug, Error)]
pub enum QueueError {
    #[error("Queue connection error: {0}")]
    Connection(String),

    #[error("Queue operation failed: {0}")]
    Operation(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Timeout waiting for message")]
    Timeout,

    #[error("Queue provider error: {0}")]
    Provider(String),
}

/// Trait for queue providers
///
/// Implementations should provide FIFO message queuing with support for
/// different backends (AWS SQS, MySQL, RabbitMQ, Kafka, etc.).
///
/// # Example
/// ```no_run
/// use flextide_core::queue::{QueueProvider, QueueMessage, QueueError};
/// use serde_json::json;
/// use std::time::Duration;
///
/// # async fn example<P: QueueProvider>(provider: &P) -> Result<(), QueueError> {
/// // Push a message
/// let payload = json!({"workflow_id": "123", "action": "execute"});
/// provider.push(payload).await?;
///
/// // Pop a message (blocks until one is available or timeout)
/// if let Some(message) = provider.pop(Some(Duration::from_secs(5))).await? {
///     println!("Received: {:?}", message);
///
///     // Acknowledge message deletion
///     if let Some(handle) = message.receipt_handle {
///         provider.delete(&handle).await?;
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub trait QueueProvider: Send + Sync {
    /// Push a JSON message to the queue
    ///
    /// # Arguments
    /// * `payload` - The JSON value to enqueue
    ///
    /// # Returns
    /// * `Ok(())` if the message was successfully enqueued
    /// * `Err(QueueError)` if the operation failed
    fn push(&self, payload: Value) -> impl Future<Output = Result<(), QueueError>> + Send;

    /// Pop a message from the queue (FIFO)
    ///
    /// This method should block until a message is available or the timeout expires.
    /// If `timeout` is `None`, it should block indefinitely.
    ///
    /// # Arguments
    /// * `timeout` - Maximum time to wait for a message. `None` means wait indefinitely.
    ///
    /// # Returns
    /// * `Ok(Some(QueueMessage))` if a message was received
    /// * `Ok(None)` if the timeout expired without receiving a message
    /// * `Err(QueueError)` if the operation failed
    fn pop(&self, timeout: Option<Duration>) -> impl Future<Output = Result<Option<QueueMessage>, QueueError>> + Send;

    /// Delete/acknowledge a message from the queue
    ///
    /// This is used to remove a message from the queue after successful processing.
    /// The receipt handle is typically obtained from `QueueMessage::receipt_handle`.
    ///
    /// # Arguments
    /// * `receipt_handle` - The receipt handle or identifier for the message to delete
    ///
    /// # Returns
    /// * `Ok(())` if the message was successfully deleted
    /// * `Err(QueueError)` if the operation failed
    fn delete(&self, receipt_handle: &str) -> impl Future<Output = Result<(), QueueError>> + Send;

    /// Get the queue name or identifier
    ///
    /// This is useful for logging and debugging purposes.
    fn queue_name(&self) -> &str;
}

