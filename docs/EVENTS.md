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
| `module_docs_area_created` | Module | A documentation area is successfully created | See [Docs Module Events](#docs-module-events) |
| `module_docs_area_updated` | Module | A documentation area is successfully updated | See [Docs Module Events](#docs-module-events) |
| `module_docs_area_deleted` | Module | A documentation area is successfully deleted | See [Docs Module Events](#docs-module-events) |
| `module_docs_folder_created` | Module | A documentation folder is successfully created | See [Docs Module Events](#docs-module-events) |
| `module_docs_folder_updated` | Module | A documentation folder is successfully updated | See [Docs Module Events](#docs-module-events) |
| `module_docs_folder_deleted` | Module | A documentation folder is successfully deleted | See [Docs Module Events](#docs-module-events) |
| `module_docs_page_created` | Module | A documentation page is successfully created | See [Docs Module Events](#docs-module-events) |
| `module_docs_page_deleted` | Module | A documentation page is successfully deleted | See [Docs Module Events](#docs-module-events) |

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

## Docs Module Events

### `module_docs_area_created`

Emitted when a new documentation area is successfully created.

**Context:**
- `organization_uuid`: The UUID of the organization that owns the area
- `user_uuid`: The UUID of the user who created the area

**Payload:**
```json
{
  "entity_type": "area",
  "entity_id": "<area_uuid>",
  "organization_uuid": "<organization_uuid>",
  "data": {
    "short_name": "<string>",
    "description": "<string | null>",
    "icon_name": "<string | null>",
    "public": "<boolean>",
    "visible": "<boolean>",
    "deletable": "<boolean>"
  }
}
```

**Payload Fields:**
- `entity_type` (string): Always `"area"`
- `entity_id` (string): The UUID of the created area
- `organization_uuid` (string): The UUID of the organization that owns the area
- `data.short_name` (string): The short name of the area
- `data.description` (string | null): The description of the area (optional)
- `data.icon_name` (string | null): The icon name for the area (optional)
- `data.public` (boolean): Whether the area is public
- `data.visible` (boolean): Whether the area is visible
- `data.deletable` (boolean): Whether the area can be deleted

**Example:**
```json
{
  "entity_type": "area",
  "entity_id": "550e8400-e29b-41d4-a716-446655440000",
  "organization_uuid": "123e4567-e89b-12d3-a456-426614174000",
  "data": {
    "short_name": "API Documentation",
    "description": "Documentation for our REST API",
    "icon_name": "book",
    "public": true,
    "visible": true,
    "deletable": true
  }
}
```

### `module_docs_area_updated`

Emitted when a documentation area is successfully updated.

**Context:**
- `organization_uuid`: The UUID of the organization that owns the area
- `user_uuid`: The UUID of the user who updated the area

**Payload:**
```json
{
  "entity_type": "area",
  "entity_id": "<area_uuid>",
  "organization_uuid": "<organization_uuid>",
  "data": {
    "short_name": "<string>",
    "description": "<string | null>",
    "icon_name": "<string | null>",
    "public": "<boolean>",
    "visible": "<boolean>",
    "deletable": "<boolean>"
  }
}
```

**Payload Fields:**
- `entity_type` (string): Always `"area"`
- `entity_id` (string): The UUID of the updated area
- `organization_uuid` (string): The UUID of the organization that owns the area
- `data.short_name` (string): The updated short name of the area
- `data.description` (string | null): The updated description of the area (optional)
- `data.icon_name` (string | null): The updated icon name for the area (optional)
- `data.public` (boolean): Whether the area is public (after update)
- `data.visible` (boolean): Whether the area is visible (after update)
- `data.deletable` (boolean): Whether the area can be deleted (after update)

**Note:** The payload contains the area data *after* the update has been applied.

### `module_docs_area_deleted`

Emitted when a documentation area is successfully deleted.

**Context:**
- `organization_uuid`: The UUID of the organization that owned the area
- `user_uuid`: The UUID of the user who deleted the area

**Payload:**
```json
{
  "entity_type": "area",
  "entity_id": "<area_uuid>",
  "organization_uuid": "<organization_uuid>",
  "data": {
    "short_name": "<string>",
    "description": "<string | null>",
    "icon_name": "<string | null>",
    "public": "<boolean>",
    "visible": "<boolean>",
    "deletable": "<boolean>"
  }
}
```

**Payload Fields:**
- `entity_type` (string): Always `"area"`
- `entity_id` (string): The UUID of the deleted area
- `organization_uuid` (string): The UUID of the organization that owned the area
- `data.short_name` (string): The short name of the area (captured before deletion)
- `data.description` (string | null): The description of the area (optional, captured before deletion)
- `data.icon_name` (string | null): The icon name for the area (optional, captured before deletion)
- `data.public` (boolean): Whether the area was public (captured before deletion)
- `data.visible` (boolean): Whether the area was visible (captured before deletion)
- `data.deletable` (boolean): Whether the area could be deleted (captured before deletion)

**Note:** The payload contains the area data *before* the deletion occurred, allowing subscribers to access the deleted entity's information.

### `module_docs_folder_created`

Emitted when a new documentation folder is successfully created.

**Context:**
- `organization_uuid`: The UUID of the organization that owns the folder
- `user_uuid`: The UUID of the user who created the folder

**Payload:**
```json
{
  "entity_type": "folder",
  "entity_id": "<folder_uuid>",
  "organization_uuid": "<organization_uuid>",
  "data": {
    "name": "<string>",
    "icon_name": "<string | null>",
    "folder_color": "<string | null>",
    "area_uuid": "<string>",
    "parent_folder_uuid": "<string | null>",
    "sort_order": "<integer>"
  }
}
```

**Payload Fields:**
- `entity_type` (string): Always `"folder"`
- `entity_id` (string): The UUID of the created folder
- `organization_uuid` (string): The UUID of the organization that owns the folder
- `data.name` (string): The name of the folder
- `data.icon_name` (string | null): The icon name for the folder (optional)
- `data.folder_color` (string | null): The hex color code for the folder (optional)
- `data.area_uuid` (string): The UUID of the area the folder belongs to
- `data.parent_folder_uuid` (string | null): The UUID of the parent folder, or null for root folders
- `data.sort_order` (integer): The sort order of the folder

**Example:**
```json
{
  "entity_type": "folder",
  "entity_id": "789e4567-e89b-12d3-a456-426614174000",
  "organization_uuid": "123e4567-e89b-12d3-a456-426614174000",
  "data": {
    "name": "Getting Started",
    "icon_name": "folder",
    "folder_color": "#3BCBB8",
    "area_uuid": "550e8400-e29b-41d4-a716-446655440000",
    "parent_folder_uuid": null,
    "sort_order": 0
  }
}
```

### `module_docs_folder_updated`

Emitted when a documentation folder is successfully updated (name change or reorder).

**Context:**
- `organization_uuid`: The UUID of the organization that owns the folder
- `user_uuid`: The UUID of the user who updated the folder

**Payload:**
```json
{
  "entity_type": "folder",
  "entity_id": "<folder_uuid>",
  "organization_uuid": "<organization_uuid>",
  "data": {
    "name": "<string>",
    "icon_name": "<string | null>",
    "folder_color": "<string | null>",
    "area_uuid": "<string>",
    "parent_folder_uuid": "<string | null>",
    "sort_order": "<integer>"
  }
}
```

**Payload Fields:**
- `entity_type` (string): Always `"folder"`
- `entity_id` (string): The UUID of the updated folder
- `organization_uuid` (string): The UUID of the organization that owns the folder
- `data.name` (string): The updated name of the folder
- `data.icon_name` (string | null): The updated icon name for the folder (optional)
- `data.folder_color` (string | null): The updated hex color code for the folder (optional)
- `data.area_uuid` (string): The UUID of the area the folder belongs to
- `data.parent_folder_uuid` (string | null): The UUID of the parent folder, or null for root folders
- `data.sort_order` (integer): The updated sort order of the folder

**Note:** The payload contains the folder data *after* the update has been applied.

### `module_docs_folder_deleted`

Emitted when a documentation folder is successfully deleted.

**Context:**
- `organization_uuid`: The UUID of the organization that owned the folder
- `user_uuid`: The UUID of the user who deleted the folder

**Payload:**
```json
{
  "entity_type": "folder",
  "entity_id": "<folder_uuid>",
  "organization_uuid": "<organization_uuid>",
  "data": {
    "name": "<string>",
    "icon_name": "<string | null>",
    "folder_color": "<string | null>",
    "area_uuid": "<string>",
    "parent_folder_uuid": "<string | null>",
    "sort_order": "<integer>"
  }
}
```

**Payload Fields:**
- `entity_type` (string): Always `"folder"`
- `entity_id` (string): The UUID of the deleted folder
- `organization_uuid` (string): The UUID of the organization that owned the folder
- `data.name` (string): The name of the folder (captured before deletion)
- `data.icon_name` (string | null): The icon name for the folder (optional, captured before deletion)
- `data.folder_color` (string | null): The hex color code for the folder (optional, captured before deletion)
- `data.area_uuid` (string): The UUID of the area the folder belonged to (captured before deletion)
- `data.parent_folder_uuid` (string | null): The UUID of the parent folder, or null for root folders (captured before deletion)
- `data.sort_order` (integer): The sort order of the folder (captured before deletion)

**Note:** The payload contains the folder data *before* the deletion occurred, allowing subscribers to access the deleted entity's information.

### `module_docs_page_created`

Emitted when a new documentation page is successfully created.

**Context:**
- `organization_uuid`: The UUID of the organization that owns the page
- `user_uuid`: The UUID of the user who created the page

**Payload:**
```json
{
  "entity_type": "page",
  "entity_id": "<page_uuid>",
  "organization_uuid": "<organization_uuid>",
  "data": {
    "title": "<string>",
    "short_summary": "<string | null>",
    "area_uuid": "<string>",
    "folder_uuid": "<string | null>",
    "parent_page_uuid": "<string | null>",
    "page_type": "<string>"
  }
}
```

**Payload Fields:**
- `entity_type` (string): Always `"page"`
- `entity_id` (string): The UUID of the created page
- `organization_uuid` (string): The UUID of the organization that owns the page
- `data.title` (string): The title of the page
- `data.short_summary` (string | null): A short summary of the page (optional)
- `data.area_uuid` (string): The UUID of the area the page belongs to
- `data.folder_uuid` (string | null): The UUID of the folder the page belongs to, or null for root pages
- `data.parent_page_uuid` (string | null): The UUID of the parent page, or null for top-level pages
- `data.page_type` (string): The type of the page

**Example:**
```json
{
  "entity_type": "page",
  "entity_id": "abc12345-e89b-12d3-a456-426614174000",
  "organization_uuid": "123e4567-e89b-12d3-a456-426614174000",
  "data": {
    "title": "Introduction to the API",
    "short_summary": "Learn the basics of our REST API",
    "area_uuid": "550e8400-e29b-41d4-a716-446655440000",
    "folder_uuid": "789e4567-e89b-12d3-a456-426614174000",
    "parent_page_uuid": null,
    "page_type": "documentation"
  }
}
```

### `module_docs_page_deleted`

Emitted when a documentation page is successfully deleted.

**Context:**
- `organization_uuid`: The UUID of the organization that owned the page
- `user_uuid`: The UUID of the user who deleted the page

**Payload:**
```json
{
  "entity_type": "page",
  "entity_id": "<page_uuid>",
  "organization_uuid": "<organization_uuid>",
  "data": {
    "title": "<string>",
    "short_summary": "<string | null>",
    "area_uuid": "<string>",
    "folder_uuid": "<string | null>",
    "parent_page_uuid": "<string | null>",
    "page_type": "<string>"
  }
}
```

**Payload Fields:**
- `entity_type` (string): Always `"page"`
- `entity_id` (string): The UUID of the deleted page
- `organization_uuid` (string): The UUID of the organization that owned the page
- `data.title` (string): The title of the page (captured before deletion)
- `data.short_summary` (string | null): A short summary of the page (optional, captured before deletion)
- `data.area_uuid` (string): The UUID of the area the page belonged to (captured before deletion)
- `data.folder_uuid` (string | null): The UUID of the folder the page belonged to, or null for root pages (captured before deletion)
- `data.parent_page_uuid` (string | null): The UUID of the parent page, or null for top-level pages (captured before deletion)
- `data.page_type` (string): The type of the page (captured before deletion)

**Note:** The payload contains the page data *before* the deletion occurred, allowing subscribers to access the deleted entity's information.

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
- `module_docs_page_updated` - When a documentation page is updated

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

