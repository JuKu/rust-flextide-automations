# CRM Module

The CRM (Customer Relationship Management) module provides functionality for managing customers, their notes, addresses, and conversations within the Flextide platform.

## Features

- Customer management with comprehensive data fields
- Customer notes with visibility controls
- Customer addresses (multiple addresses per customer)
- Conversation tracking across multiple channels
- Organization-scoped data isolation

## Usage Examples

### Loading a Customer

```rust
use flextide_modules_crm::CrmCustomer;

// Load a customer from the database
let customer = CrmCustomer::load_from_database(&pool, customer_uuid).await?;
```

### Creating a Customer

```rust
use flextide_modules_crm::{CrmCustomer, CreateCrmCustomerRequest};

let request = CreateCrmCustomerRequest {
    first_name: "John".to_string(),
    last_name: "Doe".to_string(),
    email: Some("john.doe@example.com".to_string()),
    phone_number: Some("+1234567890".to_string()),
    user_id: None,
    salutation: Some("Mr.".to_string()),
    job_title: Some("Software Engineer".to_string()),
    department: Some("Engineering".to_string()),
    company_name: Some("Example Corp".to_string()),
    fax_number: None,
    website_url: None,
    gender: None,
};

let customer_uuid = CrmCustomer::create_customer(&pool, organization_uuid, request).await?;
```

### Adding a Note to a Customer

```rust
use flextide_modules_crm::{CrmCustomer, CreateCrmCustomerNoteRequest};

// First, load the customer
let customer = CrmCustomer::load_from_database(&pool, customer_uuid).await?;

// Create a note request
let note_request = CreateCrmCustomerNoteRequest {
    note_text: "Customer called to inquire about pricing".to_string(),
    visible_to_customer: Some(false), // Optional, defaults to false
};

// Add the note to the customer
let note_uuid = customer.add_note(&pool, author_user_id, note_request).await?;
```

### Deleting a Note from a Customer

```rust
use flextide_modules_crm::CrmCustomer;

// First, load the customer
let customer = CrmCustomer::load_from_database(&pool, customer_uuid).await?;

// Delete a specific note by UUID
// This will verify that the note belongs to this customer before deleting
customer.delete_note(&pool, note_uuid).await?;
```

**Note**: The `delete_note` method verifies that the note belongs to the customer before deletion. If the note doesn't exist or doesn't belong to the customer, it will return an error.

### Listing All Notes for a Customer

```rust
use flextide_modules_crm::CrmCustomer;

// First, load the customer
let customer = CrmCustomer::load_from_database(&pool, customer_uuid).await?;

// List all notes for this customer (sorted by creation date, oldest first)
let notes = customer.list_notes(&pool).await?;

// Iterate over the notes
for note in notes {
    println!("Note: {}", note.note_text);
    println!("Author: {}", note.author_id);
    println!("Created: {}", note.created_at);
    println!("Visible to customer: {}", note.visible_to_customer);
}
```

**Note**: The `list_notes` method returns all notes for the customer, sorted by creation date in ascending order (oldest first).

### Adding an Address to a Customer

```rust
use flextide_modules_crm::{CrmCustomer, CreateCrmCustomerAddressRequest};

// First, load the customer
let customer = CrmCustomer::load_from_database(&pool, customer_uuid).await?;

// Create an address request
let address_request = CreateCrmCustomerAddressRequest {
    address_type: "Billing".to_string(), // Required: e.g., "Billing", "Shipping", "Mailing"
    street: Some("123 Main Street".to_string()),
    city: Some("New York".to_string()),
    state_province: Some("NY".to_string()),
    postal_code: Some("10001".to_string()),
    country: Some("USA".to_string()),
    is_primary: Some(true), // Optional, defaults to false
};

// Add the address to the customer
let address_uuid = customer.add_address(&pool, address_request).await?;
```

### Deleting an Address from a Customer

```rust
use flextide_modules_crm::CrmCustomer;

// First, load the customer
let customer = CrmCustomer::load_from_database(&pool, customer_uuid).await?;

// Delete a specific address by UUID
// This will verify that the address belongs to this customer before deleting
customer.delete_address(&pool, address_uuid).await?;
```

**Note**: The `delete_address` method verifies that the address belongs to the customer before deletion. If the address doesn't exist or doesn't belong to the customer, it will return an error.

### Validation

The `add_address` method includes the following validation:

- **address_type**: Must not be empty
- **is_primary**: Defaults to `false` if not specified

The `add_note` method includes the following validation:

- **author_id**: Must not be empty or null
- **note_text**: Must not be empty and must have at least 2 characters
- **visible_to_customer**: Defaults to `false` if not specified

If validation fails, the method returns a `CrmCustomerDatabaseError` with an appropriate error message.

## Database Schema

The CRM module uses the following database tables:

- `module_crm_customers` - Main customer data
- `module_crm_customer_notes` - Notes attached to customers
- `module_crm_customer_addresses` - Customer addresses
- `module_crm_conversation_channels` - Communication channels
- `module_crm_customer_conversations` - Customer conversations

All tables are prefixed with `module_crm_` to follow the module naming convention.

## Error Handling

All database operations return `Result<T, CrmCustomerDatabaseError>`, which can represent:

- Database connection errors
- SQL execution errors
- Validation errors (for `add_note` and `add_address`)

## Organization Scoping

All CRM data is scoped to organizations. When creating customers, you must provide an `organization_uuid`, and all queries automatically filter by the current organization context.

