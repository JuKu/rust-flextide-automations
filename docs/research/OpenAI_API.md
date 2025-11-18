# OpenAI API Research

## Overview

The OpenAI API provides developers with access to advanced AI models through a RESTful API interface. It offers a general-purpose "text in, text out" interface that can be applied to a wide range of language tasks without being tailored to specific use cases. The API enables integration of natural language understanding and generation capabilities into applications.

## Core API Endpoints

### Chat Completions
**Endpoint:** `POST https://api.openai.com/v1/chat/completions`

The primary endpoint for conversational AI interactions with multi-turn conversation support.

**Request Format:**
- **Method:** POST
- **Content-Type:** application/json
- **Required Parameters:**
  - `model` (string): Model identifier (e.g., "gpt-4o", "gpt-4o-mini", "gpt-4.1")
  - `messages` (array): Array of message objects with roles and content
    - Roles: "system", "user", "assistant"
    - Content: string or array (for multimodal inputs)
- **Optional Parameters:**
  - `temperature` (float, 0-2): Controls randomness (default: 1.0)
  - `top_p` (float, 0-1): Nucleus sampling parameter
  - `n` (integer): Number of completions to generate (default: 1)
  - `stream` (boolean): Enable streaming responses (default: false)
  - `stop` (string/array): Stop sequences to end generation
  - `max_tokens` (integer): Maximum tokens to generate
  - `presence_penalty` (float, -2.0 to 2.0): Penalize new topics
  - `frequency_penalty` (float, -2.0 to 2.0): Penalize repetition
  - `logit_bias` (map): Bias specific tokens
  - `user` (string): User identifier for monitoring/abuse detection
  - `response_format` (object): Force structured output (JSON schema)
  - `tools` (array): Functions/tools available to the model
  - `tool_choice` (string/object): Control tool usage ("none", "auto", or specific tool)

**Response Format:**
- **Non-streaming:** JSON object with `id`, `object`, `created`, `model`, `choices`, `usage`
- **Streaming:** Server-Sent Events (SSE) with `data:` prefixed JSON chunks
- **Response Fields:**
  - `choices`: Array of completion objects
  - `usage`: Token usage statistics (prompt_tokens, completion_tokens, total_tokens)
  - `finish_reason`: Reason for completion ("stop", "length", "tool_calls", "content_filter")

**Supported Models:**
- GPT-4o, GPT-4o Mini, GPT-4.1, GPT-4.1 Mini, GPT-4.1 Nano, GPT-4.5, GPT-5
- Reasoning models: o1, o3, o4-mini

**Modes:**
- **Non-streaming:** Standard request/response (default)
- **Streaming:** Real-time token-by-token output (set `stream: true`)
- **Function Calling:** Model can call defined functions/tools
- **Structured Outputs:** Guaranteed JSON schema compliance (GPT-4o+)
- **Multimodal:** Text, image, and audio inputs (GPT-4o, GPT-5)

### Completions (Legacy)
**Endpoint:** `POST https://api.openai.com/v1/completions`

Legacy endpoint for simple text completion without conversation context.

**Request Format:**
- **Method:** POST
- **Required Parameters:**
  - `model` (string): Legacy model identifier
  - `prompt` (string/array): Text prompt(s) to complete
- **Optional Parameters:**
  - `max_tokens`, `temperature`, `top_p`, `n`, `stream`, `stop`
  - `logprobs` (integer): Return log probabilities
  - `echo` (boolean): Echo the prompt in response
  - `suffix` (string): Text to append after completion
  - `presence_penalty`, `frequency_penalty`, `logit_bias`, `user`

**Response Format:**
- JSON object with `id`, `object`, `created`, `model`, `choices`, `usage`
- `choices` contains `text`, `index`, `logprobs`, `finish_reason`

**Status:** Deprecated in favor of Chat Completions. Not recommended for new projects.

### Embeddings
**Endpoint:** `POST https://api.openai.com/v1/embeddings`

Converts text into high-dimensional vector representations for semantic operations.

**Request Format:**
- **Method:** POST
- **Required Parameters:**
  - `model` (string): Embedding model identifier
  - `input` (string/array): Text to embed (single string or array of strings)
- **Optional Parameters:**
  - `encoding_format` (string): Output format - "float" (default) or "base64"
  - `dimensions` (integer): Reduce embedding dimensions (text-embedding-3-* only)
  - `user` (string): User identifier

**Response Format:**
- JSON object with `object`, `data` (array of embedding objects), `model`, `usage`
- Each embedding object contains `object`, `embedding` (array), `index`

**Supported Models:**
- `text-embedding-3-small`: 1536 dimensions (default), can reduce to 512-1536
- `text-embedding-3-large`: 3072 dimensions (default), can reduce to 256-3072
- `text-embedding-ada-002`: 1536 dimensions (legacy)

**Use Cases:**
- Semantic search and similarity matching
- Clustering and classification
- Recommendation systems
- Anomaly detection

### Images (DALL·E)
**Endpoint:** `POST https://api.openai.com/v1/images/generations`

Generates images from textual descriptions using DALL·E models.

**Request Format:**
- **Method:** POST
- **Required Parameters:**
  - `prompt` (string): Text description of desired image
- **Optional Parameters:**
  - `model` (string): "dall-e-2" or "dall-e-3" (default: dall-e-3)
  - `n` (integer): Number of images (1 for DALL·E 3, 1-10 for DALL·E 2)
  - `size` (string): Image dimensions
    - DALL·E 3: "1024x1024", "1792x1024", "1024x1792"
    - DALL·E 2: "256x256", "512x512", "1024x1024"
  - `quality` (string): "standard" or "hd" (DALL·E 3 only, default: standard)
  - `style` (string): "vivid" or "natural" (DALL·E 3 only, default: vivid)
  - `response_format` (string): "url" (default) or "b64_json"
  - `user` (string): User identifier

**Response Format:**
- JSON object with `created`, `data` (array of image objects)
- Each image object contains `url` or `b64_json` (base64 encoded), `revised_prompt`

**Supported Models:**
- DALL·E 3 (default, recommended)
- DALL·E 2 (legacy)

**Modes:**
- **URL Response:** Returns image URLs (default, expires after 1 hour)
- **Base64 Response:** Returns base64-encoded image data

### Audio

#### Text-to-Speech (TTS)
**Endpoint:** `POST https://api.openai.com/v1/audio/speech`

Converts text into natural-sounding speech audio.

**Request Format:**
- **Method:** POST
- **Required Parameters:**
  - `model` (string): "tts-1" or "tts-1-hd"
  - `input` (string): Text to convert to speech (max 4096 characters)
  - `voice` (string): Voice selection - "alloy", "echo", "fable", "onyx", "nova", "shimmer"
- **Optional Parameters:**
  - `response_format` (string): Audio format - "mp3", "opus", "aac", "flac" (default: mp3)
  - `speed` (float): Speech speed multiplier (0.25 to 4.0, default: 1.0)

**Response Format:**
- Binary audio file (no JSON wrapper)
- Content-Type: audio/mpeg (mp3), audio/opus, audio/aac, or audio/flac

**Supported Models:**
- `tts-1`: Faster, lower cost
- `tts-1-hd`: Higher quality, slower, higher cost

#### Speech-to-Text (Whisper)
**Endpoints:**
- **Transcriptions:** `POST https://api.openai.com/v1/audio/transcriptions`
- **Translations:** `POST https://api.openai.com/v1/audio/translations`

Converts audio into text (transcription) or translates to English (translation).

**Request Format:**
- **Method:** POST (multipart/form-data)
- **Required Parameters:**
  - `file` (file): Audio file (max 25 MB)
  - `model` (string): "whisper-1"
- **Optional Parameters:**
  - `language` (string): ISO-639-1 language code (transcriptions only)
  - `prompt` (string): Text to guide model's style/formatting
  - `response_format` (string): "json", "text", "srt", "verbose_json", "vtt" (default: json)
  - `temperature` (float): Sampling temperature (0-1)
  - `timestamp_granularities` (array): ["word", "segment"] for verbose_json

**Response Format:**
- **JSON (default):** `{"text": "transcribed text"}`
- **Text:** Plain text transcription
- **Verbose JSON:** Includes timestamps, segments, words
- **SRT/VTT:** Subtitle file formats

**Supported Models:**
- `whisper-1`: Multilingual speech recognition

**Supported Audio Formats:**
- mp3, mp4, mpeg, mpga, m4a, wav, webm

### Files Management
**Endpoints:**
- `POST https://api.openai.com/v1/files` - Upload file
- `GET https://api.openai.com/v1/files` - List files
- `GET https://api.openai.com/v1/files/{file_id}` - Retrieve file
- `GET https://api.openai.com/v1/files/{file_id}/content` - Download file content
- `DELETE https://api.openai.com/v1/files/{file_id}` - Delete file

**Purpose:** Manage files for fine-tuning, embeddings, or Assistants/Responses API.

**Upload Request:**
- **Method:** POST (multipart/form-data)
- **Required Parameters:**
  - `file` (file): File to upload
  - `purpose` (string): File purpose - "fine-tune", "assistants", "batch", "vision"

**Response Format:**
- JSON object with `id`, `object`, `bytes`, `created_at`, `filename`, `purpose`, `status`

**File Purposes:**
- `fine-tune`: Training data for fine-tuning
- `assistants`: Files for Assistants/Responses API
- `batch`: Input files for Batch API
- `vision`: Image files for vision models

### Models
**Endpoint:** `GET https://api.openai.com/v1/models`

Retrieves list of available models and their capabilities.

**Request Format:**
- **Method:** GET
- **No parameters required**

**Response Format:**
- JSON object with `object`, `data` (array of model objects)
- Each model object contains `id`, `object`, `created`, `owned_by`, `permission`, `root`, `parent`

**Use Case:** Discover available models and their properties programmatically.

### Moderations
**Endpoint:** `POST https://api.openai.com/v1/moderations`

Analyzes text for content policy violations and safety concerns.

**Request Format:**
- **Method:** POST
- **Required Parameters:**
  - `input` (string/array): Text to moderate
- **Optional Parameters:**
  - `model` (string): "text-moderation-latest" (default) or "text-moderation-stable"

**Response Format:**
- JSON object with `id`, `model`, `results` (array)
- Each result contains:
  - `flagged` (boolean): Whether content violates policy
  - `categories`: Object with boolean flags for each category
  - `category_scores`: Object with probability scores (0-1) for each category

**Categories:**
- `hate`, `hate/threatening`, `self-harm`, `sexual`, `sexual/minors`, `violence`, `violence/graphic`

**Use Cases:**
- Content moderation for user-generated content
- Safety filtering before processing
- Compliance and policy enforcement

### Batch API
**Endpoints:**
- `POST https://api.openai.com/v1/batch` - Create batch request
- `GET https://api.openai.com/v1/batch` - List batches
- `GET https://api.openai.com/v1/batch/{batch_id}` - Retrieve batch
- `POST https://api.openai.com/v1/batch/{batch_id}/cancel` - Cancel batch

**Purpose:** Asynchronous processing of large volumes of requests with 50% cost savings.

**Request Format (Create Batch):**
- **Method:** POST
- **Required Parameters:**
  - `input_file_id` (string): File ID containing requests (JSONL format)
  - `endpoint` (string): Target endpoint ("/v1/chat/completions", "/v1/embeddings", etc.)
- **Optional Parameters:**
  - `completion_window` (string): "24h" (default) or "24h"

**Batch File Format:**
- JSONL (JSON Lines) file
- Each line is a valid API request JSON object
- Must include `custom_id` for tracking

**Response Format:**
- JSON object with `id`, `object`, `endpoint`, `errors`, `input_file_id`, `completion_window`, `status`, `output_file_id`, `error_file_id`, `created_at`, `in_progress_at`, `expires_at`, `finalizing_at`, `completed_at`, `failed_at`, `expired_at`, `cancelling_at`, `cancelled_at`, `request_counts`

**Status Values:**
- `validating`, `failed`, `in_progress`, `finalizing`, `completed`, `expired`, `cancelling`, `cancelled`

**Use Cases:**
- Large-scale data processing
- Non-real-time workloads
- Cost optimization (50% discount)

### Fine-tuning
**Endpoints:**
- `POST https://api.openai.com/v1/fine_tuning/jobs` - Create fine-tuning job
- `GET https://api.openai.com/v1/fine_tuning/jobs` - List fine-tuning jobs
- `GET https://api.openai.com/v1/fine_tuning/jobs/{job_id}` - Retrieve job
- `POST https://api.openai.com/v1/fine_tuning/jobs/{job_id}/cancel` - Cancel job
- `GET https://api.openai.com/v1/fine_tuning/jobs/{job_id}/events` - List job events

**Purpose:** Customize models for specific tasks using custom training data.

**Request Format (Create Job):**
- **Method:** POST
- **Required Parameters:**
  - `training_file` (string): File ID of training data (JSONL format)
  - `model` (string): Base model to fine-tune
- **Optional Parameters:**
  - `validation_file` (string): File ID for validation data
  - `hyperparameters` (object): Training hyperparameters
    - `n_epochs` (integer): Number of epochs (default: auto)
    - `batch_size` (integer): Batch size (default: auto)
    - `learning_rate_multiplier` (float): Learning rate multiplier (default: auto)
  - `suffix` (string): Custom model name suffix
  - `integrations` (array): Third-party integrations

**Training Data Format:**
- JSONL file with examples
- Each line: `{"messages": [{"role": "system", "content": "..."}, ...]}`

**Response Format:**
- JSON object with `id`, `object`, `created_at`, `finished_at`, `model`, `fine_tuned_model`, `organization_id`, `status`, `hyperparameters`, `training_file`, `validation_file`, `result_files`, `trained_tokens`, `error`

**Status Values:**
- `validating_files`, `queued`, `running`, `succeeded`, `failed`, `cancelled`

**Supported Models:**
- GPT-4.1, GPT-4o, GPT-4o Mini
- babbage-002, davinci-002 (legacy)

**Use Cases:**
- Domain-specific language adaptation
- Style and tone customization
- Task-specific optimization

### Assistants API (Deprecated)
**Status:** Replaced by Responses API in March 2025

**Replacement:** Responses API (no additional cost beyond model usage)

**Purpose:** Previously used to build advanced AI agents capable of executing complex tasks autonomously.

### Responses API
**Status:** Introduced March 2025

**Purpose:** Advanced AI agent capabilities for complex autonomous tasks with tool integration.

**Features:**
- Stateful conversations
- Tool/function calling
- File search and code interpreter
- Persistent memory
- Multi-modal inputs

**Cost:** No additional cost beyond model usage
- File storage: $0.10 per GB per day (first GB free)
- Code Interpreter: $0.03 per session (active for 1 hour)

## Agent Capabilities and Building Agents

### Overview
The OpenAI API supports building autonomous AI agents that can perform complex, multi-step tasks by reasoning, making decisions, and using tools/functions. Agents can autonomously execute tasks, interact with external systems, and handle complex workflows.

### Function Calling / Tool Use
**Purpose:** Enable models to call external functions/tools to perform actions or retrieve information.

**Implementation:**
- Define `tools` array in Chat Completions request
- Each tool has `type` ("function"), `function` object with `name`, `description`, `parameters` (JSON schema)
- Model decides when to call tools based on conversation context
- Response includes `tool_calls` array with function name and arguments
- Execute function and send results back in next request with `tool` role message

**Tool Choice Control:**
- `tool_choice: "none"` - Model cannot call tools
- `tool_choice: "auto"` - Model decides (default)
- `tool_choice: {"type": "function", "function": {"name": "..."}}` - Force specific tool

**Agent Workflow:**
1. User sends request with tools defined
2. Model analyzes request and decides to call tool(s)
3. Response includes `tool_calls` with function name and arguments
4. Application executes function(s) and gets results
5. Send results back to model with `role: "tool"` messages
6. Model processes results and provides final response
7. Repeat steps 2-6 as needed for multi-step tasks

**Supported Built-in Tools (Responses API):**
- **Web Search:** Search the internet for current information
- **File Search:** Search through uploaded files
- **Code Interpreter:** Execute Python code in sandboxed environment
- **Computer Use:** Interact with applications and interfaces

### AgentKit
**Status:** Introduced 2025

**Purpose:** Comprehensive toolkit for building, deploying, and optimizing AI agents.

**Components:**
- **Agent Builder:** Visual interface for designing and versioning multi-agent workflows
- **ChatKit:** Toolkit for embedding customizable chat-based agent experiences
- **Evals:** Tools for measuring and improving agent performance through evaluations

**Benefits:**
- Streamlines agent development process
- Reduces complexity of creating agentic workflows
- Provides evaluation and optimization tools
- Supports multi-agent systems

### Building Agents with Chat Completions API

**Basic Agent Pattern:**
```json
{
  "model": "gpt-4o",
  "messages": [
    {"role": "system", "content": "You are a helpful assistant that can use tools."},
    {"role": "user", "content": "What's the weather in San Francisco?"}
  ],
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "get_weather",
        "description": "Get current weather for a location",
        "parameters": {
          "type": "object",
          "properties": {
            "location": {"type": "string", "description": "City name"}
          },
          "required": ["location"]
        }
      }
    }
  ],
  "tool_choice": "auto"
}
```

**Multi-Turn Agent Conversation:**
- Model may call multiple tools in sequence
- Each tool call requires a follow-up message with results
- Agent continues until task is complete or user intervenes
- Maintain conversation context across tool calls

**Best Practices:**
- Provide clear, detailed tool descriptions
- Use structured parameters (JSON schema)
- Handle tool execution errors gracefully
- Implement timeouts for long-running agents
- Log tool calls for debugging and monitoring
- Validate tool inputs before execution
- Set appropriate `max_tokens` for agent responses

### Agent Mode in ChatGPT
**Note:** This is a product feature, not an API feature, but provides context for agent capabilities.

**Description:** ChatGPT feature that enables autonomous task execution within the ChatGPT interface.

**Capabilities:**
- Navigate websites and interact with web content
- Fill out forms and edit spreadsheets
- Connect to third-party data sources
- Research and gather information autonomously
- Pause for user clarification when needed

**Availability:**
- Pro, Plus, Business, Enterprise, and Edu plans
- Monthly message limits (e.g., 40 for Plus, 400 for Pro)
- Activated via tools menu or `/agent` command

**Use Cases:**
- Event planning and coordination
- Research and information gathering
- Data entry and form filling
- Content creation and editing

## Available Models

### GPT-4 Family
- **GPT-4o** (May 2024)
  - Multimodal (text, images, audio)
  - State-of-the-art performance in voice, multilingual, and vision benchmarks
  - Pricing: $2.50 per million input tokens / $10 per million output tokens

- **GPT-4o Mini** (July 2024)
  - Cost-effective version of GPT-4o
  - Pricing: $0.15 per million input tokens / $0.60 per million output tokens

- **GPT-4.1 Family** (April 2025)
  - GPT-4.1: $2.00 input / $8.00 output per million tokens
  - GPT-4.1 Mini: $0.40 input / $1.60 output per million tokens
  - GPT-4.1 Nano: $0.10 input / $0.40 output per million tokens
  - Features: Improved coding, instruction-following, up to 1M token context

- **GPT-4.5 (Orion)** (February 2025)
  - High-performance model
  - Pricing: $75 per million input tokens / $150 per million output tokens

### GPT-5 (August 2025)
- **Features:**
  - Enhanced reasoning capabilities
  - Context window up to 1 million tokens
  - Persistent memory
  - Multimodal (text, image, audio, video)
- **Pricing:** $1.25 per million input tokens / $2.50 per million output tokens

### Reasoning Models (o-series)
- **o1:** Advanced reasoning model
- **o3:** Advanced reasoning model ($2.00 input / $8.00 output per million tokens)
- **o4-mini:** Reasoning model with multimodal capabilities (text and images)
  - Pricing: $1.10 per million input tokens / $4.40 per million output tokens

## Authentication

### API Keys
**Primary Authentication Method**

- **Format:** `sk-...` (starts with "sk-")
- **Location:** Header `Authorization: Bearer <api_key>`
- **Obtaining API Keys:**
  1. Log in to OpenAI account
  2. Navigate to API keys section in account settings
  3. Generate new secret key
  4. Store securely (key won't be displayed again)
- **Usage in HTTP Requests:**
  ```http
  Authorization: Bearer sk-...
  ```
- **Security Best Practices:**
  - Never expose in client-side code
  - Store securely in environment variables or secret management systems
  - Rotate regularly
  - Use different keys for different environments
  - Revoke compromised keys immediately
  - Never commit keys to version control
  - Monitor usage for anomalies

### Organization IDs
- **Optional Header:** `OpenAI-Organization: <org_id>`
- **Purpose:** 
  - Multi-organization account management
  - Separate billing and usage tracking per organization
  - Useful for teams and enterprises
- **Usage:**
  ```http
  Authorization: Bearer sk-...
  OpenAI-Organization: org-...
  ```

### OAuth 2.0 (Third-Party Applications)
- **Purpose:** Allow third-party applications to access OpenAI resources on behalf of users
- **Use Case:** Secure delegated access without sharing credentials directly
- **Implementation:** Standard OAuth 2.0 flow
- **Note:** Less common than API key authentication for most use cases

### Authentication in Rust (async-openai Example)
```rust
use async_openai::config::OpenAIConfig;
use async_openai::Client;

// From environment variable OPENAI_API_KEY
let config = OpenAIConfig::default();
let client = Client::with_config(config);

// Or with explicit API key
let config = OpenAIConfig::new()
    .with_api_key("sk-...")
    .with_org_id("org-..."); // Optional
let client = Client::with_config(config);
```

## Rate Limits

### Tier-Based Limits
Rate limits vary by account tier and model:
- **Free Tier:** Very limited requests per minute
- **Paid Tier:** Higher limits based on usage and subscription
- **Enterprise:** Custom rate limits with SLA guarantees

### Common Limits
- **Requests per minute (RPM):** Varies by model and tier
- **Tokens per minute (TPM):** Model-specific limits
- **Rate limit headers:** 
  - `x-ratelimit-limit-requests`
  - `x-ratelimit-remaining-requests`
  - `x-ratelimit-reset-requests`

### Handling Rate Limits
- Implement exponential backoff
- Monitor rate limit headers
- Use Batch API for non-real-time workloads
- Consider upgrading tier for higher limits

## Pricing Structure

### Token-Based Pricing
Pricing is based on per-token usage, with separate rates for input and output tokens:
- **Input tokens:** Text sent to the API
- **Output tokens:** Text generated by the API
- **Token calculation:** Approximately 4 characters per token (varies by language)

### Model Pricing (per million tokens)

**GPT-4 Family:**
- GPT-4.1: $2.00 input / $8.00 output
- GPT-4.1 Mini: $0.40 input / $1.60 output
- GPT-4.1 Nano: $0.10 input / $0.40 output
- GPT-4o: $2.50 input / $10.00 output
- GPT-4o Mini: $0.15 input / $0.60 output
- GPT-4.5: $75.00 input / $150.00 output

**GPT-5:**
- $1.25 input / $2.50 output

**Reasoning Models:**
- o3: $2.00 input / $8.00 output
- o4-mini: $1.10 input / $4.40 output

### Additional Services Pricing

**Embeddings:**
- text-embedding-3-small: $0.02 per million tokens
- text-embedding-3-large: $0.13 per million tokens

**Image Generation (DALL·E 3):**
- Standard: $0.04 per image
- HD: $0.08 per image

**Audio:**
- Text-to-Speech: $15.00 per million characters
- Whisper (Speech-to-Text): $0.006 per minute

**Fine-tuning:**
- GPT-4.1: $25.00 per million training tokens
- Mini/Nano models: $1.50–$5.00 per million tokens

**Batch API:**
- 50% discount on input/output tokens

**Assistants/Responses API:**
- No additional cost beyond model usage
- File storage: $0.10 per GB per day (first GB free)
- Code Interpreter: $0.03 per session (active for 1 hour)

## Key Features

### Streaming Responses
- **Server-Sent Events (SSE):** Real-time token streaming
- **Usage:** Set `stream: true` in request
- **Benefits:** Lower perceived latency, better UX for long responses
- **Implementation:** Handle `data:` events, parse JSON chunks

### Function Calling
- **Purpose:** Structured outputs, tool integration, and building autonomous agents
- **Usage:** Define functions/tools in request, model decides when to call
- **Benefits:** Reliable structured data, integration with external APIs, agent autonomy
- **Response Format:** Function name and arguments in JSON via `tool_calls` array
- **Agent Capabilities:** Enables multi-step autonomous task execution (see Agent Capabilities section)

### Structured Outputs
- **Purpose:** Guarantee JSON schema compliance
- **Usage:** Define response format schema
- **Benefits:** Type-safe responses, no parsing errors
- **Models:** Available on GPT-4o and newer models

### System Prompts
- **Purpose:** Control model behavior and personality
- **Usage:** Include in messages array with role "system"
- **Benefits:** Consistent behavior, role definition, guardrails

### Temperature and Sampling
- **Temperature:** Controls randomness (0.0 = deterministic, 2.0 = very creative)
- **Top-p:** Nucleus sampling parameter
- **Max tokens:** Limit response length
- **Stop sequences:** Define stopping conditions

## Error Handling

### Common Error Codes
- **401 Unauthorized:** Invalid or missing API key
- **429 Too Many Requests:** Rate limit exceeded
- **500 Internal Server Error:** OpenAI service issue
- **503 Service Unavailable:** Service overloaded

### Best Practices
- Implement retry logic with exponential backoff
- Handle rate limits gracefully
- Log errors for debugging
- Provide fallback responses for critical features
- Monitor API status page for outages

## Security Considerations

### API Key Security
- Never commit keys to version control
- Use environment variables or secret management
- Rotate keys regularly
- Use different keys per environment
- Monitor usage for anomalies

### Input Validation
- Sanitize user inputs before sending to API
- Implement content filtering
- Set appropriate usage limits
- Monitor for abuse patterns

### Output Validation
- Validate API responses before processing
- Implement content moderation
- Handle unexpected response formats
- Set timeouts for requests

## Integration Patterns

### REST API
- Standard HTTP requests
- JSON request/response format
- Stateless interactions
- Suitable for most use cases

### SDKs
- **Python:** `openai` package (official)
- **JavaScript/TypeScript:** `openai` package (official)
- **Rust:** Community-maintained crates available (no official SDK)
  - **`async-openai`** (Most Powerful & Feature-Complete):
    - Comprehensive coverage of all OpenAI API endpoints
    - Full async/await support with non-blocking operations
    - SSE streaming support
    - Ergonomic builder patterns for requests
    - Azure OpenAI Service compatibility
    - Well-documented with extensive examples
    - Actively maintained with regular updates
    - Supports all API features: Chat, Completions, Embeddings, Images, Audio, Files, Fine-tuning, Moderations
  - `openai-api`: Simple Rust library without complex async dependencies (basic features)
  - `openai-agents-rs`: Framework for building multi-agent workflows (specialized use case)
  - `ai-lib`: Unified multi-provider AI SDK supporting OpenAI and others (multi-provider abstraction)
- **Other languages:** Community-maintained SDKs available
- **Benefits:** Type safety, convenience methods, automatic retries

### Webhooks
- Not directly supported by OpenAI API
- Implement webhook pattern using external services
- Use for async processing and notifications

## Use Cases

### Enterprise Automation
- Document summarization
- Data extraction from unstructured text
- Customer interaction automation
- Report generation

### Software Engineering
- Code generation and completion
- Code refactoring
- Bug detection and fixing
- Documentation generation

### Customer-Facing Applications
- Multilingual chatbots
- Virtual assistants
- Content generation
- Translation services

### Data and Research
- Literature reviews
- Spreadsheet analysis
- Market research automation
- Data classification

### Multimodal Applications
- Image analysis and description
- Voice-activated interfaces
- Video content analysis
- Audio transcription

## Best Practices

### Cost Optimization
- Use appropriate model for task (Mini/Nano for simple tasks)
- Implement caching for repeated queries
- Use Batch API for non-real-time workloads
- Monitor token usage and optimize prompts
- Set max_tokens to limit response length

### Performance
- Use streaming for better perceived latency
- Implement request queuing for high-volume applications
- Cache embeddings for repeated queries
- Use appropriate model size for task complexity
- Monitor and optimize prompt length

### Reliability
- Implement retry logic with exponential backoff
- Handle rate limits gracefully
- Use fallback models when appropriate
- Monitor API status and health
- Implement circuit breakers for critical paths

### Development
- Use structured outputs for type safety
- Implement comprehensive error handling
- Log requests/responses for debugging
- Use environment-specific API keys
- Test with various input scenarios

## Limitations and Considerations

### Context Window
- Varies by model (8K to 1M tokens)
- Input + output must fit within limit
- Consider chunking for long documents
- Monitor token usage to avoid truncation

### Latency
- Response time varies by model and complexity
- Streaming reduces perceived latency
- Consider async processing for non-critical tasks
- Implement timeouts and fallbacks

### Rate Limits
- Vary by account tier and model
- Can impact high-volume applications
- Consider Batch API or queue system
- Monitor and plan for scaling

### Cost Management
- Token costs can accumulate quickly
- Monitor usage and set budgets
- Use appropriate models for tasks
- Implement usage tracking and alerts

## References

### Official Documentation
- [OpenAI API Documentation](https://platform.openai.com/docs)
- [OpenAI API Overview](https://platform.openai.com/docs/overview)
- [OpenAI API Reference Introduction](https://platform.openai.com/docs/api-reference/introduction)
- [OpenAI API Reference](https://platform.openai.com/docs/api-reference)
- [OpenAI Pricing](https://openai.com/pricing)
- [OpenAI Help Center](https://help.openai.com/en/collections/3675931-api)

### Model Information
- [GPT-4o Wikipedia](https://en.wikipedia.org/wiki/GPT-4o)
- [GPT-4.5 Wikipedia](https://en.wikipedia.org/wiki/GPT-4.5)
- [OpenAI Products and Applications](https://en.wikipedia.org/wiki/Products_and_applications_of_OpenAI)

### Resources and Guides
- [OpenAI Cookbook](https://cookbook.openai.com)
- [OpenAI Python SDK](https://github.com/openai/openai-python)
- [OpenAI JavaScript SDK](https://github.com/openai/openai-node)
- [New Tools for Building Agents](https://openai.com/index/new-tools-for-building-agents/)
- [Introducing AgentKit](https://openai.com/index/introducing-agentkit/)

### Rust SDKs (Community-Maintained)
- **[async-openai](https://github.com/64bit/async-openai)** - Most powerful and feature-complete async Rust library
  - Comprehensive API coverage, SSE streaming, builder patterns, Azure compatibility
- [openai-api](https://github.com/openai-rs/openai-api) - Simple Rust library for OpenAI API
- [openai-agents-rs](https://lib.rs/crates/openai-agents-rs) - Multi-agent workflows framework
- [ai-lib](https://www.ailib.info/) - Unified multi-provider AI SDK

### News and Updates
- [OpenAI Blog - API Announcement](https://openai.com/blog/openai-api)
- [Responses API Launch (Reuters)](https://www.reuters.com/technology/artificial-intelligence/openai-launches-new-developer-tools-chinese-ai-startups-gain-ground-2025-03-11)
- [GPT-4.1 Family Announcement](https://www.techradar.com/news/live/openai-chatgpt-announcements-april-2025)
- [GPT-5 Release](https://cincodias.elpais.com/smartlife/lifestyle/2025-08-07/openai-gpt-5-oficial-mejoras.html)

### Pricing Resources
- [OpenAI API Pricing Deep Dive](https://www.hostking.dev/openai-api-pricing)
- [OpenAI Features, Pricing, and Use Cases](https://www.walturn.com/insights/what-is-openai-features-pricing-and-use-cases)
- [OpenAI API Pricing Calculator](https://theaibasic.com/openai-api-pricing-calculator)

### Video Tutorials
- [OpenAI Structured Outputs Tutorial](https://www.youtube.com/watch?v=6e_oFG4JVg8)
- [OpenAI API Pricing & Access Tutorial](https://www.youtube.com/watch?v=wVlj1WJmDiQ)

### Agent Resources
- [ChatGPT Agent Mode Help](https://help.openai.com/en/articles/11752874-chatgpt-agent)
- [OpenAI AgentKit Documentation](https://openai.com/index/introducing-agentkit/)

