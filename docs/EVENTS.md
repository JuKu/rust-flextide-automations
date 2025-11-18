# Flextide Event System - Event Reference

This document provides a comprehensive reference of all events emitted by the Flextide platform, including their names, payloads, and when they are triggered.

## Event Naming Conventions

Events follow a consistent naming pattern:
- **Core events**: `core_<event_name>`
- **Module events**: `module_<module_name>_<event_name>`
- **Plugin events**: `plugin_<plugin_name>_<event_name>`
- **Integration events**: `integration_<integration_name>_<event_name>`

## Event Context

All events include the following context (when available):
- `organization_uuid`: The UUID of the organization that owns the entity
- `user_uuid`: The UUID of the user who triggered the action
- `timestamp`: Automatically set when the event is emitted

## Event Reference Table

| Event Name | Type | Triggered When | Payload Structure |
|------------|------|----------------|-------------------|
| `core_organization_created` | Core | An organization is successfully created | See [Core Events](#core-events) |
| `module_crm_customer_created` | Module | A customer is successfully created in the CRM module | See [CRM Module Events](#crm-module-events) |
| `module_crm_customer_updated` | Module | A customer is successfully updated in the CRM module | See [CRM Module Events](#crm-module-events) |
| `module_crm_customer_deleted` | Module | A customer is successfully deleted from the CRM module | See [CRM Module Events](#crm-module-events) |

## Core Events

### `core_organization_created`

Emitted when a new organization is successfully created.

**Context:**
- `organization_uuid`: The UUID of the newly created organization
- `user_uuid`: The UUID of the user who created the organization

**Payload:**
```json
{
  "entity_type": "organization",
  "entity_id": "<organization_uuid>",
  "data": {
    "name": "<organization_name>",
    "owner_user_id": "<user_uuid>"
  }
}
```

**Payload Fields:**
- `entity_type` (string): Always `"organization"`
- `entity_id` (string): The UUID of the created organization
- `data.name` (string): The name of the organization
- `data.owner_user_id` (string): The UUID of the user who owns the organization

**Example:**
```json
{
  "entity_type": "organization",
  "entity_id": "550e8400-e29b-41d4-a716-446655440000",
  "data": {
    "name": "Acme Corporation",
    "owner_user_id": "123e4567-e89b-12d3-a456-426614174000"
  }
}
```

## CRM Module Events

### `module_crm_customer_created`

Emitted when a new customer is successfully created in the CRM module.

**Context:**
- `organization_uuid`: The UUID of the organization that owns the customer
- `user_uuid`: The UUID of the user who created the customer

**Payload:**
```json
{
  "entity_type": "customer",
  "entity_id": "<customer_uuid>",
  "data": {
    "first_name": "<string>",
    "last_name": "<string>",
    "email": "<string | null>",
    "company_name": "<string | null>"
  }
}
```

**Payload Fields:**
- `entity_type` (string): Always `"customer"`
- `entity_id` (string): The UUID of the created customer
- `data.first_name` (string): The customer's first name
- `data.last_name` (string): The customer's last name
- `data.email` (string | null): The customer's email address (optional)
- `data.company_name` (string | null): The customer's company name (optional)

**Example:**
```json
{
  "entity_type": "customer",
  "entity_id": "789e4567-e89b-12d3-a456-426614174000",
  "data": {
    "first_name": "John",
    "last_name": "Doe",
    "email": "john.doe@example.com",
    "company_name": "Doe Industries"
  }
}
```

### `module_crm_customer_updated`

Emitted when a customer is successfully updated in the CRM module.

**Context:**
- `organization_uuid`: The UUID of the organization that owns the customer
- `user_uuid`: The UUID of the user who updated the customer

**Payload:**
```json
{
  "entity_type": "customer",
  "entity_id": "<customer_uuid>",
  "data": {
    "first_name": "<string>",
    "last_name": "<string>",
    "email": "<string | null>",
    "company_name": "<string | null>"
  }
}
```

**Payload Fields:**
- `entity_type` (string): Always `"customer"`
- `entity_id` (string): The UUID of the updated customer
- `data.first_name` (string): The customer's updated first name
- `data.last_name` (string): The customer's updated last name
- `data.email` (string | null): The customer's updated email address (optional)
- `data.company_name` (string | null): The customer's updated company name (optional)

**Note:** The payload contains the customer data *after* the update has been applied.

**Example:**
```json
{
  "entity_type": "customer",
  "entity_id": "789e4567-e89b-12d3-a456-426614174000",
  "data": {
    "first_name": "Jane",
    "last_name": "Doe",
    "email": "jane.doe@example.com",
    "company_name": "Doe Industries"
  }
}
```

### `module_crm_customer_deleted`

Emitted when a customer is successfully deleted from the CRM module.

**Context:**
- `organization_uuid`: The UUID of the organization that owned the customer
- `user_uuid`: The UUID of the user who deleted the customer

**Payload:**
```json
{
  "entity_type": "customer",
  "entity_id": "<customer_uuid>",
  "data": {
    "first_name": "<string>",
    "last_name": "<string>",
    "email": "<string | null>",
    "company_name": "<string | null>"
  }
}
```

**Payload Fields:**
- `entity_type` (string): Always `"customer"`
- `entity_id` (string): The UUID of the deleted customer
- `data.first_name` (string): The customer's first name (captured before deletion)
- `data.last_name` (string): The customer's last name (captured before deletion)
- `data.email` (string | null): The customer's email address (optional, captured before deletion)
- `data.company_name` (string | null): The customer's company name (optional, captured before deletion)

**Note:** The payload contains the customer data *before* the deletion occurred, allowing subscribers to access the deleted entity's information.

**Example:**
```json
{
  "entity_type": "customer",
  "entity_id": "789e4567-e89b-12d3-a456-426614174000",
  "data": {
    "first_name": "John",
    "last_name": "Doe",
    "email": "john.doe@example.com",
    "company_name": "Doe Industries"
  }
}
```

## Future Events

The following events are planned but not yet implemented:

### Core Events (Planned)
- `core_user_created` - When a user account is created
- `core_user_updated` - When a user account is updated
- `core_user_deleted` - When a user account is deleted
- `core_organization_updated` - When an organization is updated
- `core_organization_deleted` - When an organization is deleted

### CRM Module Events (Planned)
- `module_crm_customer_note_created` - When a note is added to a customer
- `module_crm_customer_note_updated` - When a customer note is updated
- `module_crm_customer_note_deleted` - When a customer note is deleted
- `module_crm_customer_address_created` - When an address is added to a customer
- `module_crm_customer_address_deleted` - When a customer address is deleted
- `module_crm_customer_conversation_created` - When a conversation is created for a customer

### Docs Module Events (Planned)
- `module_docs_document_created` - When a document is created
- `module_docs_document_updated` - When a document is updated
- `module_docs_document_deleted` - When a document is deleted

### Project Management Module Events (Planned)
- `module_project_management_project_created` - When a project is created
- `module_project_management_project_updated` - When a project is updated
- `module_project_management_project_deleted` - When a project is deleted

### Integration Events (Planned)
- `integration_jira_project_synced` - When a Jira project is synced
- `integration_slack_message_sent` - When a Slack message is sent

## Subscribing to Events

To subscribe to events, you can:

1. **Database-backed subscriptions**: Create a subscription in the `event_subscriptions` table
2. **Runtime subscriptions**: Register an `EventSubscriber` implementation programmatically

For more information on the event system architecture and usage, see:
- `.cursor/rules/event_system.mdc` - Event system rules and conventions
- `backend/crates/flextide-core/src/events/README.md` - Technical documentation

## Event Payload Standard Structure

All events follow a standard payload structure:

```json
{
  "entity_type": "<string>",
  "entity_id": "<uuid>",
  "data": {
    // Entity-specific fields
  }
}
```

- `entity_type`: A string identifying the type of entity (e.g., "organization", "customer")
- `entity_id`: The UUID of the entity that triggered the event
- `data`: An object containing entity-specific data relevant to the event

This structure ensures consistency across all events and makes it easier to build generic event handlers.

