-- Create workflows, runs, and queue_messages tables
-- Supports both MySQL and PostgreSQL
-- Note: updated_at is managed by application code, not database triggers
--
-- This migration creates:
-- 1. workflows: Stores workflow definitions (directed graphs of nodes and edges)
-- 2. runs: Tracks workflow execution runs (groups all node executions for a single workflow trigger)
-- 3. queue_messages: Database-based queue for distributing workflow execution tasks
--    across multiple worker instances. Messages are processed in FIFO order with
--    support for concurrent worker access using SELECT ... FOR UPDATE SKIP LOCKED.

-- ============================================================================
-- WORKFLOWS TABLE
-- ============================================================================
-- Workflows are directed graphs of nodes and edges that define automation
-- processes. Each workflow belongs to an organization and contains:
-- - Graph structure (nodes and edges) stored as JSON
-- - Metadata (name, description, status)
-- - Execution settings

CREATE TABLE IF NOT EXISTS workflows (
    -- Primary key (UUID for consistency with other tables)
    uuid CHAR(36) NOT NULL PRIMARY KEY,
    
    -- Organization this workflow belongs to
    organization_uuid CHAR(36) NOT NULL,
    
    -- Workflow metadata
    name VARCHAR(255) NOT NULL,
    description TEXT,
    
    -- Workflow definition (JSON containing nodes and edges)
    -- Structure:
    -- {
    --   "nodes": [
    --     {
    --       "id": "node-id",
    --       "type": "node-type",
    --       "position": { "x": 100, "y": 200 },
    --       "data": { ... },
    --       "config": { ... }
    --     }
    --   ],
    --   "edges": [
    --     {
    --       "id": "edge-id",
    --       "source": "node-id",
    --       "target": "node-id",
    --       "sourceHandle": "pin-name",
    --       "targetHandle": "pin-name"
    --     }
    --   ]
    -- }
    definition JSON NOT NULL,
    
    -- Workflow status: 'active', 'paused', 'draft', 'error'
    -- active: Workflow is enabled and can be triggered
    -- paused: Workflow is temporarily disabled
    -- draft: Workflow is being edited, not ready for execution
    -- error: Workflow has errors and cannot execute
    status VARCHAR(20) NOT NULL DEFAULT 'draft' 
        CHECK (status IN ('active', 'paused', 'draft', 'error')),
    
    -- User who created this workflow
    created_by CHAR(36) NOT NULL,
    
    -- Timestamps
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign keys
    FOREIGN KEY (organization_uuid) REFERENCES organizations(uuid) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users(uuid) ON DELETE RESTRICT
);

-- Indexes for workflows table
CREATE INDEX IF NOT EXISTS idx_workflows_organization 
    ON workflows(organization_uuid);

CREATE INDEX IF NOT EXISTS idx_workflows_status 
    ON workflows(status);

CREATE INDEX IF NOT EXISTS idx_workflows_created_by 
    ON workflows(created_by);

CREATE INDEX IF NOT EXISTS idx_workflows_org_status 
    ON workflows(organization_uuid, status);

CREATE INDEX IF NOT EXISTS idx_workflows_org_name 
    ON workflows(organization_uuid, name);

-- ============================================================================
-- RUNS TABLE
-- ============================================================================
-- Tracks workflow execution runs. Each run represents a single execution of a workflow,
-- triggered by an event (webhook, cron, manual, etc.). A run groups all node executions
-- (queue_messages) that belong to the same workflow execution instance.

CREATE TABLE IF NOT EXISTS runs (
    -- Primary key (UUID for consistency with other tables)
    uuid CHAR(36) NOT NULL PRIMARY KEY,
    
    -- Workflow this run belongs to
    workflow_id CHAR(36) NOT NULL,
    
    -- Organization (denormalized for efficient filtering)
    organization_uuid CHAR(36) NOT NULL,
    
    -- Run status: 'not_started', 'running', 'completed', 'failed', 'cancelled', 'waiting', 'blocked'
    -- not_started: Workflow execution has not started yet
    -- running: Workflow is currently executing
    -- completed: All nodes executed successfully
    -- failed: Workflow execution failed (one or more nodes failed)
    -- cancelled: Workflow execution was cancelled
    -- waiting: Workflow is waiting for a condition (e.g., wait node)
    -- blocked: Workflow execution is blocked (e.g., waiting for external resource)
    status VARCHAR(20) NOT NULL DEFAULT 'not_started'
        CHECK (status IN ('not_started', 'running', 'completed', 'failed', 'cancelled', 'waiting', 'blocked')),
    
    -- How this run was triggered
    -- 'manual': Triggered manually by user
    -- 'webhook': Triggered by webhook
    -- 'cron': Triggered by schedule
    -- 'event': Triggered by event
    -- 'subworkflow': Triggered by parent workflow
    trigger_type VARCHAR(255) NOT NULL DEFAULT 'manual',
    
    -- User who triggered this run (if manual) or system identifier
    triggered_by CHAR(36),
    
    -- Execution timestamps
    started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    finished_at TIMESTAMP NULL,
    
    -- Error information (if run failed)
    error_message TEXT,
    error_code VARCHAR(100),
    
    -- Execution metadata (JSON for flexible storage)
    -- Can store trigger data, execution context, etc.
    metadata JSON,
    
    -- Timestamps
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign keys
    FOREIGN KEY (workflow_id) REFERENCES workflows(uuid) ON DELETE CASCADE,
    FOREIGN KEY (organization_uuid) REFERENCES organizations(uuid) ON DELETE CASCADE,
    FOREIGN KEY (triggered_by) REFERENCES users(uuid) ON DELETE SET NULL
);

-- Indexes for runs table
CREATE INDEX IF NOT EXISTS idx_runs_workflow 
    ON runs(workflow_id);

CREATE INDEX IF NOT EXISTS idx_runs_organization 
    ON runs(organization_uuid);

CREATE INDEX IF NOT EXISTS idx_runs_status 
    ON runs(status);

CREATE INDEX IF NOT EXISTS idx_runs_started_at 
    ON runs(started_at);

CREATE INDEX IF NOT EXISTS idx_runs_workflow_status 
    ON runs(workflow_id, status);

CREATE INDEX IF NOT EXISTS idx_runs_org_status 
    ON runs(organization_uuid, status);

CREATE INDEX IF NOT EXISTS idx_runs_triggered_by 
    ON runs(triggered_by);

-- ============================================================================
-- PERMISSIONS: Add can_see_last_executions permission
-- ============================================================================
-- Add permission for viewing execution history
-- This permission belongs to the 'workflows' permission group

INSERT INTO permissions (id, name, title, description, visible, sort_order, permission_group_name)
SELECT new_permissions.id, new_permissions.name, new_permissions.title, new_permissions.description, new_permissions.visible, new_permissions.sort_order, new_permissions.permission_group_name
FROM (
    VALUES
        ('30000000-0000-0000-0000-000000000001', 'can_see_last_executions', 'Can see last executions', 'The user is able to see the execution history and last executions', 1, 10, 'workflows')
) AS new_permissions(id, name, title, description, visible, sort_order, permission_group_name)
WHERE NOT EXISTS (SELECT 1 FROM permissions WHERE permissions.name = new_permissions.name);

-- ============================================================================
-- QUEUE_MESSAGES TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS queue_messages (
    -- Primary key (UUID for consistency with other tables)
    id CHAR(36) NOT NULL PRIMARY KEY,
    
    -- Workflow execution context
    -- workflow_id: Identifies which workflow this message belongs to
    --               Used to load workflow graph and determine next nodes
    workflow_id CHAR(36) NOT NULL,
    
    -- run_id: Identifies a single workflow execution run
    --         All messages with the same run_id belong to the same execution
    --         Foreign key to runs table
    run_id CHAR(36) NOT NULL,
    
    -- Message payload (JSON containing NodeExecutionRequest)
    -- Contains: input, config, context (workflow_id, run_id, node_id, execution_id)
    payload JSON NOT NULL,
    
    -- Message status: 'pending', 'processing', 'completed', 'failed', 'dead_letter'
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    
    -- Priority (lower number = higher priority, default 0)
    -- Allows priority queues in the future
    priority INTEGER NOT NULL DEFAULT 0,
    
    -- Receipt handle for acknowledgment (generated when message is claimed)
    -- Used as the receipt_handle in QueueMessage for delete() operations
    receipt_handle CHAR(36),
    
    -- Visibility timeout: when this message becomes visible again if not deleted
    -- Used to handle crashed workers (message reappears after timeout)
    -- Default: message is immediately visible
    visible_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Retry tracking
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    
    -- Timestamps
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    processed_at TIMESTAMP NULL,
    
    -- Error tracking (for failed messages)
    error_message TEXT,
    error_code VARCHAR(100),
    
    -- Optional: Queue name/partition for multi-queue support
    queue_name VARCHAR(100) NOT NULL DEFAULT 'default',
    
    -- Foreign keys
    FOREIGN KEY (workflow_id) REFERENCES workflows(uuid) ON DELETE CASCADE,
    FOREIGN KEY (run_id) REFERENCES runs(uuid) ON DELETE CASCADE
);

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Main index for pop() query: filters by status, orders by visible_at, priority, created_at
-- This index enables efficient FIFO retrieval with concurrent worker access
CREATE INDEX IF NOT EXISTS idx_queue_messages_status_visible_priority 
    ON queue_messages(status, visible_at, priority, created_at);

-- Index for tracking workflow execution (all messages in a single run)
-- Useful for monitoring, debugging, and cleanup operations
CREATE INDEX IF NOT EXISTS idx_queue_messages_workflow_run 
    ON queue_messages(workflow_id, run_id);

-- Index for receipt_handle lookup (used in delete() operations)
CREATE INDEX IF NOT EXISTS idx_queue_messages_receipt_handle 
    ON queue_messages(receipt_handle);

-- Index for queue name (supports multi-queue scenarios)
CREATE INDEX IF NOT EXISTS idx_queue_messages_queue_name 
    ON queue_messages(queue_name);

-- Index for cleanup operations (finding old completed messages)
CREATE INDEX IF NOT EXISTS idx_queue_messages_status_processed 
    ON queue_messages(status, processed_at);
