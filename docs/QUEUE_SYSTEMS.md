# Queue Systems for Flextide Workflow Execution

## Overview

This document analyzes queue systems suitable for implementing the workflow execution queue in Flextide. The queue is used to distribute workflow execution tasks across multiple worker instances, enabling horizontal scaling and reliable task processing.

## Requirements

Based on the platform architecture and requirements:

- **FIFO Ordering**: Workflow tasks should be processed in order when possible
- **Reliability**: Messages must not be lost (at-least-once delivery acceptable)
- **Acknowledgment**: Support for message acknowledgment/deletion after processing
- **Scalability**: Support horizontal scaling with multiple workers
- **Deployment Flexibility**: Support both self-hosted and cloud deployments
- **Low Latency**: Fast message delivery for responsive workflow execution
- **Simple Integration**: Easy to implement with the existing `QueueProvider` trait

## Queue System Analysis

### Tier 1: Recommended for MVP / Core Implementation

#### 1. **Database-Based Queue (MySQL/PostgreSQL)**

**Status**: ✅ **HIGHEST PRIORITY** - Already supported infrastructure

**Pros:**
- Zero additional infrastructure (uses existing database)
- Perfect for self-hosted deployments
- ACID guarantees for reliability
- Easy to implement with existing `sqlx` dependencies
- Built-in persistence and durability
- Simple to debug and monitor (SQL queries)
- Supports FIFO via `ORDER BY` and `LIMIT`
- Transactional message deletion

**Cons:**
- Lower throughput than dedicated message queues
- Database connection overhead
- Polling-based (not event-driven)
- Can create database load under high message volume

**Implementation Notes:**
- Use `SELECT ... FOR UPDATE SKIP LOCKED` for concurrent worker access
- Table structure: `id`, `payload` (JSON), `created_at`, `processed_at`, `status`
- Index on `status` and `created_at` for efficient polling
- Supports both MySQL and PostgreSQL (already in stack)

**Use Cases:**
- Self-hosted deployments
- Small to medium workloads
- Development and testing
- Organizations preferring minimal infrastructure

**Priority**: **P0 - Implement First**

---

#### 2. **AWS SQS (Simple Queue Service)**

**Status**: ✅ **HIGH PRIORITY** - Cloud-native, widely used

**Pros:**
- Fully managed (no infrastructure to maintain)
- High availability and durability (99.999999999% durability)
- Automatic scaling
- Dead-letter queue support for failed messages
- FIFO queues available (exactly-once processing)
- Pay-per-use pricing model
- Excellent AWS ecosystem integration
- Long polling support (reduces API calls)

**Cons:**
- AWS-specific (vendor lock-in)
- Requires AWS SDK dependency
- Message size limit (256 KB)
- Visibility timeout complexity
- Cost can add up at high volumes

**Implementation Notes:**
- Use FIFO queues for ordered processing
- Standard queues for higher throughput (if ordering not critical)
- Receipt handles for message deletion
- Long polling (up to 20 seconds) reduces empty responses

**Use Cases:**
- Cloud deployments on AWS
- High-volume production workloads
- Organizations already using AWS

**Priority**: **P1 - Implement for Cloud**

---

### Tier 2: Recommended for Production / Enterprise

#### 3. **RabbitMQ**

**Status**: ✅ **MEDIUM PRIORITY** - Industry standard, feature-rich

**Pros:**
- Mature, battle-tested message broker
- Rich feature set (routing, exchanges, bindings)
- Excellent management UI
- Supports multiple protocols (AMQP, MQTT, STOMP)
- Clustering and high availability
- Message persistence options
- Good documentation and community

**Cons:**
- Requires separate infrastructure/service
- More complex setup than database queue
- Erlang-based (different tech stack)
- Memory-intensive
- Requires operational expertise

**Implementation Notes:**
- Use durable queues for reliability
- Acknowledge messages after processing
- Prefetch count for worker concurrency control
- Dead-letter exchanges for failed messages

**Use Cases:**
- Enterprise self-hosted deployments
- Complex routing requirements (future)
- Organizations with existing RabbitMQ infrastructure

**Priority**: **P2 - Implement for Enterprise**

---

#### 4. **Redis Streams / Redis Lists**

**Status**: ✅ **MEDIUM PRIORITY** - Fast, simple, in-memory

**Pros:**
- Extremely fast (in-memory)
- Simple API
- Built-in pub/sub capabilities
- Redis Streams provide message ordering and consumer groups
- Widely available (managed services from all cloud providers)
- Can be used for caching as well (dual purpose)

**Cons:**
- In-memory (data loss risk if not persisted)
- Requires separate infrastructure
- Redis Streams API is newer (less mature ecosystem)
- Memory limits (need to monitor)

**Implementation Notes:**
- Use Redis Streams for FIFO ordering and consumer groups
- Enable AOF (Append-Only File) persistence for durability
- XREADGROUP for worker consumer groups
- XACK for message acknowledgment

**Use Cases:**
- High-throughput scenarios
- Low-latency requirements
- Organizations already using Redis
- Development/testing environments

**Priority**: **P2 - Implement for Performance**

---

### Tier 3: Specialized / Future Consideration

#### 5. **Apache Kafka**

**Status**: ⚠️ **LOW PRIORITY** - Overkill for current needs

**Pros:**
- Extremely high throughput
- Distributed, fault-tolerant
- Event streaming capabilities
- Long-term message retention
- Excellent for event sourcing patterns

**Cons:**
- Complex setup and operations
- Overkill for simple task queues
- Requires Zookeeper/KRaft
- High resource requirements
- Steep learning curve
- Better suited for event streaming than task queues

**Use Cases:**
- Future event streaming features
- Very high-volume scenarios (millions of messages/sec)
- Event sourcing architecture

**Priority**: **P3 - Future Consideration Only**

---

#### 6. **NATS / NATS JetStream**

**Status**: ⚠️ **LOW PRIORITY** - Lightweight alternative

**Pros:**
- Lightweight and fast
- Simple deployment
- Good for cloud-native architectures
- JetStream provides persistence
- Low resource footprint

**Cons:**
- Less mature ecosystem than RabbitMQ
- Smaller community
- Less feature-rich than RabbitMQ
- JetStream is newer (less battle-tested)

**Use Cases:**
- Cloud-native deployments
- Organizations preferring lightweight solutions
- Microservices architectures

**Priority**: **P3 - Alternative to RabbitMQ**

---

#### 7. **Google Cloud Pub/Sub**

**Status**: ⚠️ **LOW PRIORITY** - GCP-specific

**Pros:**
- Fully managed
- High availability
- Global message routing
- At-least-once delivery guarantees
- Good GCP integration

**Cons:**
- GCP-specific (vendor lock-in)
- Less common than SQS
- Different API patterns

**Use Cases:**
- GCP deployments
- Organizations using Google Cloud

**Priority**: **P3 - GCP Alternative to SQS**

---

#### 8. **Azure Service Bus**

**Status**: ⚠️ **LOW PRIORITY** - Azure-specific

**Pros:**
- Fully managed
- Enterprise features (dead-letter, sessions)
- Good Azure integration
- FIFO queues available

**Cons:**
- Azure-specific (vendor lock-in)
- Less common than SQS
- More complex API

**Use Cases:**
- Azure deployments
- Organizations using Microsoft stack

**Priority**: **P3 - Azure Alternative to SQS**

---

## Implementation Priority

### Phase 1: MVP (Immediate)
1. **Database Queue (MySQL/PostgreSQL)** - P0
   - Zero additional infrastructure
   - Works for all deployment scenarios
   - Fast to implement

### Phase 2: Cloud Support (3-6 months)
2. **AWS SQS** - P1
   - Most common cloud queue
   - Managed service benefits

### Phase 3: Enterprise Features (6-12 months)
3. **RabbitMQ** - P2
   - Enterprise self-hosted option
   - Rich feature set

4. **Redis Streams** - P2
   - High-performance option
   - Simple integration

### Phase 4: Specialized (12+ months)
5. **Kafka** - P3
   - Only if event streaming needed

6. **NATS / GCP Pub/Sub / Azure Service Bus** - P3
   - Based on customer demand

## Recommended Implementation Order

1. **Start with Database Queue** - Provides immediate value with zero infrastructure overhead
2. **Add AWS SQS** - Enables cloud deployments and managed service benefits
3. **Add RabbitMQ** - Provides enterprise self-hosted option with rich features
4. **Add Redis Streams** - Provides high-performance option for latency-sensitive workloads

## Design Considerations

### Queue Provider Trait
The existing `QueueProvider` trait is well-designed and supports all these implementations:
- `push()` - Enqueue messages
- `pop()` - Dequeue with timeout
- `delete()` - Acknowledge/delete messages
- `queue_name()` - For logging/debugging

### Worker Implementation Pattern
All queue providers should support the same worker pattern:
```rust
loop {
    if let Some(message) = queue.pop(timeout).await? {
        // Process workflow instruction
        process_workflow(message.payload).await?;
        
        // Acknowledge message
        if let Some(handle) = message.receipt_handle {
            queue.delete(&handle).await?;
        }
    }
}
```

### Configuration
Queue provider selection should be configurable via environment variable:
- `QUEUE_PROVIDER=database` (default)
- `QUEUE_PROVIDER=sqs`
- `QUEUE_PROVIDER=rabbitmq`
- `QUEUE_PROVIDER=redis`

## Conclusion

For Flextide's workflow automation platform, the recommended queue systems are:

1. **Database Queue** - Essential for MVP and self-hosted deployments
2. **AWS SQS** - Essential for cloud deployments
3. **RabbitMQ** - Recommended for enterprise features
4. **Redis Streams** - Recommended for high-performance scenarios

This provides a comprehensive solution covering:
- ✅ Self-hosted deployments (Database, RabbitMQ, Redis)
- ✅ Cloud deployments (AWS SQS)
- ✅ Simple setups (Database)
- ✅ Enterprise features (RabbitMQ)
- ✅ High performance (Redis)

The database queue should be implemented first as it requires no additional infrastructure and works for all deployment scenarios.

