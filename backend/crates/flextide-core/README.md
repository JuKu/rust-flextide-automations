# Flextide Core

The Flextide Core library provides core functionality for the Flextide workflow automation platform, including database abstractions, user management, password hashing, and validation utilities.

## Modules

### Database Module

Provides database connection pool abstractions supporting MySQL, PostgreSQL, and SQLite.

### User Module

Provides user management functionality including:
- User creation and retrieval
- Password hashing and verification (using Argon2)
- Email and password validation
- Organization membership verification

## User Management

### Checking User Organization Membership

The `user_belongs_to_organization` function allows you to verify if a specific user belongs to a specific organization. This is essential for authorization checks in multi-tenant applications.

#### Function Signature

```rust
pub async fn user_belongs_to_organization(
    pool: &DatabasePool,
    user_uuid: &str,
    organization_uuid: &str,
) -> Result<bool, UserDatabaseError>
```

#### Parameters

- **`pool`**: Database connection pool (`DatabasePool`)
- **`user_uuid`**: UUID of the user to check (string)
- **`organization_uuid`**: UUID of the organization to check (string)

#### Returns

- **`Ok(true)`**: User belongs to the organization
- **`Ok(false)`**: User does not belong to the organization
- **`Err(UserDatabaseError)`**: Database query failed

#### Usage Example

```rust
use flextide_core::user::user_belongs_to_organization;
use flextide_core::database::DatabasePool;

// Check if a user belongs to an organization
match user_belongs_to_organization(&pool, user_uuid, organization_uuid).await {
    Ok(true) => {
        // User is a member of the organization
        println!("User {} belongs to organization {}", user_uuid, organization_uuid);
    }
    Ok(false) => {
        // User is not a member
        println!("User {} does not belong to organization {}", user_uuid, organization_uuid);
    }
    Err(e) => {
        // Database error occurred
        eprintln!("Error checking membership: {}", e);
    }
}
```

#### Implementation Details

- Queries the `organization_members` table to check for membership
- Uses `COUNT(*)` to efficiently check for existence
- Supports MySQL, PostgreSQL, and SQLite databases
- Returns a boolean result wrapped in `Result` for error handling

#### Database Schema

The function queries the `organization_members` table:

```sql
CREATE TABLE organization_members (
    org_id CHAR(36) NOT NULL,
    user_id CHAR(36) NOT NULL,
    role VARCHAR(20) NOT NULL DEFAULT 'member',
    joined_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (org_id, user_id),
    FOREIGN KEY (org_id) REFERENCES organizations(uuid) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(uuid) ON DELETE CASCADE
);
```

The function checks for the existence of a row where both `user_id` and `org_id` match the provided UUIDs.

#### Error Handling

The function returns `UserDatabaseError` which can represent:
- **Database connection errors**: Connection pool issues
- **SQL execution errors**: Query execution failures

#### Use Cases

This function is commonly used for:
- **API Middleware**: Verifying user access to organization-scoped resources
- **Authorization Checks**: Ensuring users can only access data from their organizations
- **Multi-tenant Security**: Enforcing data isolation between organizations

#### Example: API Middleware Integration

```rust
use flextide_core::user::user_belongs_to_organization;

// In your API middleware
async fn organization_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    // Extract user UUID and organization UUID from request
    let user_uuid = extract_user_uuid(&request);
    let org_uuid = extract_org_uuid(&request);
    
    // Check membership
    match user_belongs_to_organization(&state.db_pool, &user_uuid, &org_uuid).await {
        Ok(true) => {
            // User belongs to organization, proceed with request
            next.run(request).await
        }
        Ok(false) => {
            // User does not belong, return 403 Forbidden
            error_response(
                StatusCode::FORBIDDEN,
                json!({ "error": "User does not belong to this organization" }),
            )
        }
        Err(e) => {
            // Database error, return 500
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Database error" }),
            )
        }
    }
}
```

## Other User Functions

### Get User by Email

```rust
use flextide_core::user::get_user_by_email;

let user = get_user_by_email(&pool, "user@example.com").await?;
```

### Password Hashing and Verification

```rust
use flextide_core::user::{hash_password, verify_password};

// Hash a password
let hash = hash_password("my_secure_password")?;

// Verify a password
let is_valid = verify_password("my_secure_password", &hash)?;
```

### Email and Password Validation

```rust
use flextide_core::user::{validate_email, validate_password};

// Validate email format
validate_email("user@example.com")?;

// Validate password strength
validate_password("secure_password_123")?;
```

## Database Support

The core library supports multiple database backends:
- **MySQL**: Full support
- **PostgreSQL**: Full support
- **SQLite**: Full support

All database operations are abstracted through the `DatabasePool` enum, allowing the same code to work with any supported database.

## Error Handling

All functions return `Result` types for proper error handling:
- `UserDatabaseError`: Database-related errors
- `PasswordError`: Password hashing/verification errors
- `EmailValidationError`: Email validation errors
- `PasswordValidationError`: Password validation errors

## License

Part of the Flextide workflow automation platform.

