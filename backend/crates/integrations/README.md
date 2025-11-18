# Integrations Crate

This crate provides integration clients for external APIs and services that can be used by nodes in the Flextide workflow automation platform.

## Structure

- `openai/` - OpenAI API client for chat completions, embeddings, and other OpenAI services

## Usage

### OpenAI Integration

```rust
use integrations::OpenAIClient;
use integrations::openai::{ChatCompletionRequest, ChatMessage, MessageRole};

let client = OpenAIClient::new("your-api-key".to_string());

let request = ChatCompletionRequest {
    model: "gpt-4o".to_string(),
    messages: vec![
        ChatMessage {
            role: MessageRole::System,
            content: "You are a helpful assistant.".to_string(),
        },
        ChatMessage {
            role: MessageRole::User,
            content: "Hello!".to_string(),
        },
    ],
    temperature: Some(0.7),
    max_tokens: Some(100),
    stream: Some(false),
};

let response = client.chat_completion(request).await?;
```

## Adding New Integrations

To add a new integration:

1. Create a new module directory under `src/` (e.g., `src/anthropic/`)
2. Implement the client following the pattern used in `openai/`
3. Export the client and types from `src/lib.rs`
4. Add appropriate error types and request/response types

