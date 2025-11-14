//! Database operations for CRM customers

use crate::customer::{
    CreateCrmCustomerAddressRequest, CreateCrmCustomerConversationRequest,
    CreateCrmCustomerNoteRequest, CreateCrmCustomerRequest, CrmCustomer, CrmCustomerConversation,
    CrmCustomerNote, UpdateCrmCustomerRequest,
};
use chrono::{DateTime, Utc};
use flextide_core::database::{DatabaseError, DatabasePool};
use sqlx::Row;
use thiserror::Error;

/// Error type for CRM customer database operations
#[derive(Debug, Error)]
pub enum CrmCustomerDatabaseError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("SQL execution error: {0}")]
    Sql(#[from] sqlx::Error),

    #[error("Author ID cannot be empty")]
    EmptyAuthorId,

    #[error("Note text cannot be empty and must be at least 2 characters long")]
    InvalidNoteText,

    #[error("Address type cannot be empty")]
    EmptyAddressType,
}

/// Load a customer from the database by UUID
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `customer_uuid` - UUID of the customer to load
///
/// # Errors
/// Returns `CrmCustomerDatabaseError` if the database query fails or customer is not found
pub async fn load_customer_by_uuid(
    pool: &DatabasePool,
    customer_uuid: &str,
) -> Result<CrmCustomer, CrmCustomerDatabaseError> {
    match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query(
                "SELECT uuid, organization_uuid, first_name, last_name, email, phone_number, 
                 user_id, salutation, job_title, department, company_name, fax_number, 
                 website_url, gender, created_at, updated_at 
                 FROM module_crm_customers WHERE uuid = ?",
            )
            .bind(customer_uuid)
            .fetch_optional(p)
            .await?;

            match row {
                Some(row) => Ok(CrmCustomer {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    first_name: row.get("first_name"),
                    last_name: row.get("last_name"),
                    email: row.get::<Option<String>, _>("email"),
                    phone_number: row.get::<Option<String>, _>("phone_number"),
                    user_id: row.get::<Option<String>, _>("user_id"),
                    salutation: row.get::<Option<String>, _>("salutation"),
                    job_title: row.get::<Option<String>, _>("job_title"),
                    department: row.get::<Option<String>, _>("department"),
                    company_name: row.get::<Option<String>, _>("company_name"),
                    fax_number: row.get::<Option<String>, _>("fax_number"),
                    website_url: row.get::<Option<String>, _>("website_url"),
                    gender: row.get::<Option<String>, _>("gender"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
                }),
                None => Err(CrmCustomerDatabaseError::Sql(sqlx::Error::RowNotFound)),
            }
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT uuid, organization_uuid, first_name, last_name, email, phone_number, 
                 user_id, salutation, job_title, department, company_name, fax_number, 
                 website_url, gender, created_at, updated_at 
                 FROM module_crm_customers WHERE uuid = $1",
            )
            .bind(customer_uuid)
            .fetch_optional(p)
            .await?;

            match row {
                Some(row) => Ok(CrmCustomer {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    first_name: row.get("first_name"),
                    last_name: row.get("last_name"),
                    email: row.get::<Option<String>, _>("email"),
                    phone_number: row.get::<Option<String>, _>("phone_number"),
                    user_id: row.get::<Option<String>, _>("user_id"),
                    salutation: row.get::<Option<String>, _>("salutation"),
                    job_title: row.get::<Option<String>, _>("job_title"),
                    department: row.get::<Option<String>, _>("department"),
                    company_name: row.get::<Option<String>, _>("company_name"),
                    fax_number: row.get::<Option<String>, _>("fax_number"),
                    website_url: row.get::<Option<String>, _>("website_url"),
                    gender: row.get::<Option<String>, _>("gender"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
                }),
                None => Err(CrmCustomerDatabaseError::Sql(sqlx::Error::RowNotFound)),
            }
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT uuid, organization_uuid, first_name, last_name, email, phone_number, 
                 user_id, salutation, job_title, department, company_name, fax_number, 
                 website_url, gender, created_at, updated_at 
                 FROM module_crm_customers WHERE uuid = ?1",
            )
            .bind(customer_uuid)
            .fetch_optional(p)
            .await?;

            match row {
                Some(row) => Ok(CrmCustomer {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    first_name: row.get("first_name"),
                    last_name: row.get("last_name"),
                    email: row.get::<Option<String>, _>("email"),
                    phone_number: row.get::<Option<String>, _>("phone_number"),
                    user_id: row.get::<Option<String>, _>("user_id"),
                    salutation: row.get::<Option<String>, _>("salutation"),
                    job_title: row.get::<Option<String>, _>("job_title"),
                    department: row.get::<Option<String>, _>("department"),
                    company_name: row.get::<Option<String>, _>("company_name"),
                    fax_number: row.get::<Option<String>, _>("fax_number"),
                    website_url: row.get::<Option<String>, _>("website_url"),
                    gender: row.get::<Option<String>, _>("gender"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
                }),
                None => Err(CrmCustomerDatabaseError::Sql(sqlx::Error::RowNotFound)),
            }
        }
    }
}

/// Create a new customer in the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `organization_uuid` - UUID of the organization the customer belongs to
/// * `request` - Customer creation request
///
/// # Returns
/// Returns the UUID of the newly created customer
///
/// # Errors
/// Returns `CrmCustomerDatabaseError` if the database operation fails
pub async fn create_customer(
    pool: &DatabasePool,
    organization_uuid: &str,
    request: CreateCrmCustomerRequest,
) -> Result<String, CrmCustomerDatabaseError> {
    let customer_uuid = uuid::Uuid::new_v4().to_string();
    let now = Utc::now();

    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query(
                "INSERT INTO module_crm_customers 
                 (uuid, organization_uuid, first_name, last_name, email, phone_number, 
                  user_id, salutation, job_title, department, company_name, fax_number, 
                  website_url, gender, created_at, updated_at) 
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&customer_uuid)
            .bind(organization_uuid)
            .bind(&request.first_name)
            .bind(&request.last_name)
            .bind(&request.email)
            .bind(&request.phone_number)
            .bind(&request.user_id)
            .bind(&request.salutation)
            .bind(&request.job_title)
            .bind(&request.department)
            .bind(&request.company_name)
            .bind(&request.fax_number)
            .bind(&request.website_url)
            .bind(&request.gender)
            .bind(now)
            .bind(now)
            .execute(p)
            .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO module_crm_customers 
                 (uuid, organization_uuid, first_name, last_name, email, phone_number, 
                  user_id, salutation, job_title, department, company_name, fax_number, 
                  website_url, gender, created_at, updated_at) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)",
            )
            .bind(&customer_uuid)
            .bind(organization_uuid)
            .bind(&request.first_name)
            .bind(&request.last_name)
            .bind(&request.email)
            .bind(&request.phone_number)
            .bind(&request.user_id)
            .bind(&request.salutation)
            .bind(&request.job_title)
            .bind(&request.department)
            .bind(&request.company_name)
            .bind(&request.fax_number)
            .bind(&request.website_url)
            .bind(&request.gender)
            .bind(now)
            .bind(now)
            .execute(p)
            .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO module_crm_customers 
                 (uuid, organization_uuid, first_name, last_name, email, phone_number, 
                  user_id, salutation, job_title, department, company_name, fax_number, 
                  website_url, gender, created_at, updated_at) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            )
            .bind(&customer_uuid)
            .bind(organization_uuid)
            .bind(&request.first_name)
            .bind(&request.last_name)
            .bind(&request.email)
            .bind(&request.phone_number)
            .bind(&request.user_id)
            .bind(&request.salutation)
            .bind(&request.job_title)
            .bind(&request.department)
            .bind(&request.company_name)
            .bind(&request.fax_number)
            .bind(&request.website_url)
            .bind(&request.gender)
            .bind(now)
            .bind(now)
            .execute(p)
            .await?;
        }
    }

    Ok(customer_uuid)
}

/// Delete a customer from the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `customer_uuid` - UUID of the customer to delete
///
/// # Errors
/// Returns `CrmCustomerDatabaseError` if the database operation fails or customer is not found
///
/// # Note
/// This will cascade delete all related records (notes, addresses, conversations) due to foreign key constraints
pub async fn delete_customer(
    pool: &DatabasePool,
    customer_uuid: &str,
) -> Result<(), CrmCustomerDatabaseError> {
    match pool {
        DatabasePool::MySql(p) => {
            let result = sqlx::query("DELETE FROM module_crm_customers WHERE uuid = ?")
                .bind(customer_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(CrmCustomerDatabaseError::Sql(sqlx::Error::RowNotFound));
            }
        }
        DatabasePool::Postgres(p) => {
            let result = sqlx::query("DELETE FROM module_crm_customers WHERE uuid = $1")
                .bind(customer_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(CrmCustomerDatabaseError::Sql(sqlx::Error::RowNotFound));
            }
        }
        DatabasePool::Sqlite(p) => {
            let result = sqlx::query("DELETE FROM module_crm_customers WHERE uuid = ?1")
                .bind(customer_uuid)
                .execute(p)
                .await?;

            if result.rows_affected() == 0 {
                return Err(CrmCustomerDatabaseError::Sql(sqlx::Error::RowNotFound));
            }
        }
    }

    Ok(())
}

/// Create a new customer note in the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `customer_uuid` - UUID of the customer the note belongs to
/// * `author_id` - UUID of the user creating the note
/// * `request` - Note creation request
///
/// # Returns
/// Returns the UUID of the newly created note
///
/// # Errors
/// Returns `CrmCustomerDatabaseError` if the database operation fails
pub async fn create_customer_note(
    pool: &DatabasePool,
    customer_uuid: &str,
    author_id: &str,
    request: CreateCrmCustomerNoteRequest,
) -> Result<String, CrmCustomerDatabaseError> {
    let note_uuid = uuid::Uuid::new_v4().to_string();
    let now = Utc::now();
    // Default to false if not specified
    let visible_to_customer = if request.visible_to_customer.unwrap_or(false) {
        1
    } else {
        0
    };

    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query(
                "INSERT INTO module_crm_customer_notes 
                 (uuid, customer_uuid, note_text, author_id, visible_to_customer, created_at, updated_at) 
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&note_uuid)
            .bind(customer_uuid)
            .bind(&request.note_text)
            .bind(author_id)
            .bind(visible_to_customer)
            .bind(now)
            .bind(now)
            .execute(p)
            .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO module_crm_customer_notes 
                 (uuid, customer_uuid, note_text, author_id, visible_to_customer, created_at, updated_at) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(&note_uuid)
            .bind(customer_uuid)
            .bind(&request.note_text)
            .bind(author_id)
            .bind(visible_to_customer)
            .bind(now)
            .bind(now)
            .execute(p)
            .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO module_crm_customer_notes 
                 (uuid, customer_uuid, note_text, author_id, visible_to_customer, created_at, updated_at) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            )
            .bind(&note_uuid)
            .bind(customer_uuid)
            .bind(&request.note_text)
            .bind(author_id)
            .bind(visible_to_customer)
            .bind(now)
            .bind(now)
            .execute(p)
            .await?;
        }
    }

    Ok(note_uuid)
}

/// Delete a customer note from the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `customer_uuid` - UUID of the customer the note belongs to (for verification)
/// * `note_uuid` - UUID of the note to delete
///
/// # Returns
/// Returns `Ok(())` if the note was successfully deleted
///
/// # Errors
/// Returns `CrmCustomerDatabaseError` if:
/// - The note does not exist
/// - The note does not belong to the specified customer
/// - The database operation fails
pub async fn delete_customer_note(
    pool: &DatabasePool,
    customer_uuid: &str,
    note_uuid: &str,
) -> Result<(), CrmCustomerDatabaseError> {
    match pool {
        DatabasePool::MySql(p) => {
            let result = sqlx::query(
                "DELETE FROM module_crm_customer_notes 
                 WHERE uuid = ? AND customer_uuid = ?",
            )
            .bind(note_uuid)
            .bind(customer_uuid)
            .execute(p)
            .await?;

            if result.rows_affected() == 0 {
                return Err(CrmCustomerDatabaseError::Sql(sqlx::Error::RowNotFound));
            }
        }
        DatabasePool::Postgres(p) => {
            let result = sqlx::query(
                "DELETE FROM module_crm_customer_notes 
                 WHERE uuid = $1 AND customer_uuid = $2",
            )
            .bind(note_uuid)
            .bind(customer_uuid)
            .execute(p)
            .await?;

            if result.rows_affected() == 0 {
                return Err(CrmCustomerDatabaseError::Sql(sqlx::Error::RowNotFound));
            }
        }
        DatabasePool::Sqlite(p) => {
            let result = sqlx::query(
                "DELETE FROM module_crm_customer_notes 
                 WHERE uuid = ?1 AND customer_uuid = ?2",
            )
            .bind(note_uuid)
            .bind(customer_uuid)
            .execute(p)
            .await?;

            if result.rows_affected() == 0 {
                return Err(CrmCustomerDatabaseError::Sql(sqlx::Error::RowNotFound));
            }
        }
    }

    Ok(())
}

/// Load all notes for a customer from the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `customer_uuid` - UUID of the customer to load notes for
///
/// # Returns
/// Returns a vector of `CrmCustomerNote` sorted by creation date (oldest first)
///
/// # Errors
/// Returns `CrmCustomerDatabaseError` if the database query fails
pub async fn load_customer_notes(
    pool: &DatabasePool,
    customer_uuid: &str,
) -> Result<Vec<CrmCustomerNote>, CrmCustomerDatabaseError> {
    match pool {
        DatabasePool::MySql(p) => {
            let rows = sqlx::query(
                "SELECT uuid, customer_uuid, note_text, author_id, visible_to_customer, 
                 created_at, updated_at 
                 FROM module_crm_customer_notes 
                 WHERE customer_uuid = ? 
                 ORDER BY created_at ASC",
            )
            .bind(customer_uuid)
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .map(|row| {
                    let visible_to_customer_int: i64 = row.get("visible_to_customer");
                    CrmCustomerNote {
                        uuid: row.get("uuid"),
                        customer_uuid: row.get("customer_uuid"),
                        note_text: row.get("note_text"),
                        author_id: row.get("author_id"),
                        visible_to_customer: visible_to_customer_int != 0,
                        created_at: row.get::<DateTime<Utc>, _>("created_at"),
                        updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
                    }
                })
                .collect())
        }
        DatabasePool::Postgres(p) => {
            let rows = sqlx::query(
                "SELECT uuid, customer_uuid, note_text, author_id, visible_to_customer, 
                 created_at, updated_at 
                 FROM module_crm_customer_notes 
                 WHERE customer_uuid = $1 
                 ORDER BY created_at ASC",
            )
            .bind(customer_uuid)
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .map(|row| {
                    let visible_to_customer_int: i64 = row.get("visible_to_customer");
                    CrmCustomerNote {
                        uuid: row.get("uuid"),
                        customer_uuid: row.get("customer_uuid"),
                        note_text: row.get("note_text"),
                        author_id: row.get("author_id"),
                        visible_to_customer: visible_to_customer_int != 0,
                        created_at: row.get::<DateTime<Utc>, _>("created_at"),
                        updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
                    }
                })
                .collect())
        }
        DatabasePool::Sqlite(p) => {
            let rows = sqlx::query(
                "SELECT uuid, customer_uuid, note_text, author_id, visible_to_customer, 
                 created_at, updated_at 
                 FROM module_crm_customer_notes 
                 WHERE customer_uuid = ?1 
                 ORDER BY created_at ASC",
            )
            .bind(customer_uuid)
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .map(|row| {
                    let visible_to_customer_int: i64 = row.get("visible_to_customer");
                    CrmCustomerNote {
                        uuid: row.get("uuid"),
                        customer_uuid: row.get("customer_uuid"),
                        note_text: row.get("note_text"),
                        author_id: row.get("author_id"),
                        visible_to_customer: visible_to_customer_int != 0,
                        created_at: row.get::<DateTime<Utc>, _>("created_at"),
                        updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
                    }
                })
                .collect())
        }
    }
}

/// Create a new customer address in the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `customer_uuid` - UUID of the customer the address belongs to
/// * `request` - Address creation request
///
/// # Returns
/// Returns the UUID of the newly created address
///
/// # Errors
/// Returns `CrmCustomerDatabaseError` if the database operation fails
pub async fn create_customer_address(
    pool: &DatabasePool,
    customer_uuid: &str,
    request: CreateCrmCustomerAddressRequest,
) -> Result<String, CrmCustomerDatabaseError> {
    let address_uuid = uuid::Uuid::new_v4().to_string();
    let now = Utc::now();
    // Default to false if not specified
    let is_primary = if request.is_primary.unwrap_or(false) { 1 } else { 0 };

    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query(
                "INSERT INTO module_crm_customer_addresses 
                 (uuid, customer_uuid, address_type, street, city, state_province, 
                  postal_code, country, is_primary, created_at, updated_at) 
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&address_uuid)
            .bind(customer_uuid)
            .bind(&request.address_type)
            .bind(&request.street)
            .bind(&request.city)
            .bind(&request.state_province)
            .bind(&request.postal_code)
            .bind(&request.country)
            .bind(is_primary)
            .bind(now)
            .bind(now)
            .execute(p)
            .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO module_crm_customer_addresses 
                 (uuid, customer_uuid, address_type, street, city, state_province, 
                  postal_code, country, is_primary, created_at, updated_at) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
            )
            .bind(&address_uuid)
            .bind(customer_uuid)
            .bind(&request.address_type)
            .bind(&request.street)
            .bind(&request.city)
            .bind(&request.state_province)
            .bind(&request.postal_code)
            .bind(&request.country)
            .bind(is_primary)
            .bind(now)
            .bind(now)
            .execute(p)
            .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO module_crm_customer_addresses 
                 (uuid, customer_uuid, address_type, street, city, state_province, 
                  postal_code, country, is_primary, created_at, updated_at) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            )
            .bind(&address_uuid)
            .bind(customer_uuid)
            .bind(&request.address_type)
            .bind(&request.street)
            .bind(&request.city)
            .bind(&request.state_province)
            .bind(&request.postal_code)
            .bind(&request.country)
            .bind(is_primary)
            .bind(now)
            .bind(now)
            .execute(p)
            .await?;
        }
    }

    Ok(address_uuid)
}

/// Delete a customer address from the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `customer_uuid` - UUID of the customer the address belongs to (for verification)
/// * `address_uuid` - UUID of the address to delete
///
/// # Returns
/// Returns `Ok(())` if the address was successfully deleted
///
/// # Errors
/// Returns `CrmCustomerDatabaseError` if:
/// - The address does not exist
/// - The address does not belong to the specified customer
/// - The database operation fails
pub async fn delete_customer_address(
    pool: &DatabasePool,
    customer_uuid: &str,
    address_uuid: &str,
) -> Result<(), CrmCustomerDatabaseError> {
    match pool {
        DatabasePool::MySql(p) => {
            let result = sqlx::query(
                "DELETE FROM module_crm_customer_addresses 
                 WHERE uuid = ? AND customer_uuid = ?",
            )
            .bind(address_uuid)
            .bind(customer_uuid)
            .execute(p)
            .await?;

            if result.rows_affected() == 0 {
                return Err(CrmCustomerDatabaseError::Sql(sqlx::Error::RowNotFound));
            }
        }
        DatabasePool::Postgres(p) => {
            let result = sqlx::query(
                "DELETE FROM module_crm_customer_addresses 
                 WHERE uuid = $1 AND customer_uuid = $2",
            )
            .bind(address_uuid)
            .bind(customer_uuid)
            .execute(p)
            .await?;

            if result.rows_affected() == 0 {
                return Err(CrmCustomerDatabaseError::Sql(sqlx::Error::RowNotFound));
            }
        }
        DatabasePool::Sqlite(p) => {
            let result = sqlx::query(
                "DELETE FROM module_crm_customer_addresses 
                 WHERE uuid = ?1 AND customer_uuid = ?2",
            )
            .bind(address_uuid)
            .bind(customer_uuid)
            .execute(p)
            .await?;

            if result.rows_affected() == 0 {
                return Err(CrmCustomerDatabaseError::Sql(sqlx::Error::RowNotFound));
            }
        }
    }

    Ok(())
}

/// Search customers by query string
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `organization_uuid` - UUID of the organization to search customers in
/// * `query` - Search query string (searches in first_name, last_name, email, company_name, phone_number, job_title)
///
/// # Returns
/// Returns a vector of `CrmCustomer` matching the search query
///
/// # Errors
/// Returns `CrmCustomerDatabaseError` if the database query fails
pub async fn search_customers(
    pool: &DatabasePool,
    organization_uuid: &str,
    query: &str,
) -> Result<Vec<CrmCustomer>, CrmCustomerDatabaseError> {
    let search_pattern = format!("%{}%", query.trim());
    
    match pool {
        DatabasePool::MySql(p) => {
            let rows = sqlx::query(
                "SELECT uuid, organization_uuid, first_name, last_name, email, phone_number, 
                 user_id, salutation, job_title, department, company_name, fax_number, 
                 website_url, gender, created_at, updated_at 
                 FROM module_crm_customers 
                 WHERE organization_uuid = ? 
                 AND (
                     first_name LIKE ? 
                     OR last_name LIKE ? 
                     OR email LIKE ? 
                     OR company_name LIKE ? 
                     OR phone_number LIKE ? 
                     OR job_title LIKE ?
                 )
                 ORDER BY last_name ASC, first_name ASC",
            )
            .bind(organization_uuid)
            .bind(&search_pattern)
            .bind(&search_pattern)
            .bind(&search_pattern)
            .bind(&search_pattern)
            .bind(&search_pattern)
            .bind(&search_pattern)
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .map(|row| CrmCustomer {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    first_name: row.get("first_name"),
                    last_name: row.get("last_name"),
                    email: row.get::<Option<String>, _>("email"),
                    phone_number: row.get::<Option<String>, _>("phone_number"),
                    user_id: row.get::<Option<String>, _>("user_id"),
                    salutation: row.get::<Option<String>, _>("salutation"),
                    job_title: row.get::<Option<String>, _>("job_title"),
                    department: row.get::<Option<String>, _>("department"),
                    company_name: row.get::<Option<String>, _>("company_name"),
                    fax_number: row.get::<Option<String>, _>("fax_number"),
                    website_url: row.get::<Option<String>, _>("website_url"),
                    gender: row.get::<Option<String>, _>("gender"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
                })
                .collect())
        }
        DatabasePool::Postgres(p) => {
            let rows = sqlx::query(
                "SELECT uuid, organization_uuid, first_name, last_name, email, phone_number, 
                 user_id, salutation, job_title, department, company_name, fax_number, 
                 website_url, gender, created_at, updated_at 
                 FROM module_crm_customers 
                 WHERE organization_uuid = $1 
                 AND (
                     first_name ILIKE $2 
                     OR last_name ILIKE $2 
                     OR email ILIKE $2 
                     OR company_name ILIKE $2 
                     OR phone_number ILIKE $2 
                     OR job_title ILIKE $2
                 )
                 ORDER BY last_name ASC, first_name ASC",
            )
            .bind(organization_uuid)
            .bind(&search_pattern)
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .map(|row| CrmCustomer {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    first_name: row.get("first_name"),
                    last_name: row.get("last_name"),
                    email: row.get::<Option<String>, _>("email"),
                    phone_number: row.get::<Option<String>, _>("phone_number"),
                    user_id: row.get::<Option<String>, _>("user_id"),
                    salutation: row.get::<Option<String>, _>("salutation"),
                    job_title: row.get::<Option<String>, _>("job_title"),
                    department: row.get::<Option<String>, _>("department"),
                    company_name: row.get::<Option<String>, _>("company_name"),
                    fax_number: row.get::<Option<String>, _>("fax_number"),
                    website_url: row.get::<Option<String>, _>("website_url"),
                    gender: row.get::<Option<String>, _>("gender"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
                })
                .collect())
        }
        DatabasePool::Sqlite(p) => {
            let rows = sqlx::query(
                "SELECT uuid, organization_uuid, first_name, last_name, email, phone_number, 
                 user_id, salutation, job_title, department, company_name, fax_number, 
                 website_url, gender, created_at, updated_at 
                 FROM module_crm_customers 
                 WHERE organization_uuid = ?1 
                 AND (
                     first_name LIKE ?2 
                     OR last_name LIKE ?2 
                     OR email LIKE ?2 
                     OR company_name LIKE ?2 
                     OR phone_number LIKE ?2 
                     OR job_title LIKE ?2
                 )
                 ORDER BY last_name ASC, first_name ASC",
            )
            .bind(organization_uuid)
            .bind(&search_pattern)
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .map(|row| CrmCustomer {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    first_name: row.get("first_name"),
                    last_name: row.get("last_name"),
                    email: row.get::<Option<String>, _>("email"),
                    phone_number: row.get::<Option<String>, _>("phone_number"),
                    user_id: row.get::<Option<String>, _>("user_id"),
                    salutation: row.get::<Option<String>, _>("salutation"),
                    job_title: row.get::<Option<String>, _>("job_title"),
                    department: row.get::<Option<String>, _>("department"),
                    company_name: row.get::<Option<String>, _>("company_name"),
                    fax_number: row.get::<Option<String>, _>("fax_number"),
                    website_url: row.get::<Option<String>, _>("website_url"),
                    gender: row.get::<Option<String>, _>("gender"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
                })
                .collect())
        }
    }
}

/// List customers for an organization with pagination
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `organization_uuid` - UUID of the organization to list customers for
/// * `page` - Page number (1-based)
/// * `page_size` - Number of customers per page (max 50)
///
/// # Returns
/// Returns a tuple of (customers, total_count)
///
/// # Errors
/// Returns `CrmCustomerDatabaseError` if the database query fails
pub async fn list_customers_paginated(
    pool: &DatabasePool,
    organization_uuid: &str,
    page: u32,
    page_size: u32,
) -> Result<(Vec<CrmCustomer>, u32), CrmCustomerDatabaseError> {
    // Ensure page_size doesn't exceed 50
    let page_size = page_size.min(50);
    let offset = (page.saturating_sub(1)) * page_size;
    
    // Get total count
    let total_count = match pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM module_crm_customers WHERE organization_uuid = ?")
                .bind(organization_uuid)
                .fetch_one(p)
                .await?;
            let count: i64 = row.get("count");
            count as u32
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM module_crm_customers WHERE organization_uuid = $1")
                .bind(organization_uuid)
                .fetch_one(p)
                .await?;
            let count: i64 = row.get("count");
            count as u32
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM module_crm_customers WHERE organization_uuid = ?1")
                .bind(organization_uuid)
                .fetch_one(p)
                .await?;
            let count: i64 = row.get("count");
            count as u32
        }
    };
    
    // Get paginated customers
    let customers = match pool {
        DatabasePool::MySql(p) => {
            let rows = sqlx::query(
                "SELECT uuid, organization_uuid, first_name, last_name, email, phone_number, 
                 user_id, salutation, job_title, department, company_name, fax_number, 
                 website_url, gender, created_at, updated_at 
                 FROM module_crm_customers 
                 WHERE organization_uuid = ? 
                 ORDER BY last_name ASC, first_name ASC 
                 LIMIT ? OFFSET ?"
            )
            .bind(organization_uuid)
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(p)
            .await?;
            
            rows.into_iter()
                .map(|row| CrmCustomer {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    first_name: row.get("first_name"),
                    last_name: row.get("last_name"),
                    email: row.get::<Option<String>, _>("email"),
                    phone_number: row.get::<Option<String>, _>("phone_number"),
                    user_id: row.get::<Option<String>, _>("user_id"),
                    salutation: row.get::<Option<String>, _>("salutation"),
                    job_title: row.get::<Option<String>, _>("job_title"),
                    department: row.get::<Option<String>, _>("department"),
                    company_name: row.get::<Option<String>, _>("company_name"),
                    fax_number: row.get::<Option<String>, _>("fax_number"),
                    website_url: row.get::<Option<String>, _>("website_url"),
                    gender: row.get::<Option<String>, _>("gender"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
                })
                .collect()
        }
        DatabasePool::Postgres(p) => {
            let rows = sqlx::query(
                "SELECT uuid, organization_uuid, first_name, last_name, email, phone_number, 
                 user_id, salutation, job_title, department, company_name, fax_number, 
                 website_url, gender, created_at, updated_at 
                 FROM module_crm_customers 
                 WHERE organization_uuid = $1 
                 ORDER BY last_name ASC, first_name ASC 
                 LIMIT $2 OFFSET $3"
            )
            .bind(organization_uuid)
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(p)
            .await?;
            
            rows.into_iter()
                .map(|row| CrmCustomer {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    first_name: row.get("first_name"),
                    last_name: row.get("last_name"),
                    email: row.get::<Option<String>, _>("email"),
                    phone_number: row.get::<Option<String>, _>("phone_number"),
                    user_id: row.get::<Option<String>, _>("user_id"),
                    salutation: row.get::<Option<String>, _>("salutation"),
                    job_title: row.get::<Option<String>, _>("job_title"),
                    department: row.get::<Option<String>, _>("department"),
                    company_name: row.get::<Option<String>, _>("company_name"),
                    fax_number: row.get::<Option<String>, _>("fax_number"),
                    website_url: row.get::<Option<String>, _>("website_url"),
                    gender: row.get::<Option<String>, _>("gender"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
                })
                .collect()
        }
        DatabasePool::Sqlite(p) => {
            let rows = sqlx::query(
                "SELECT uuid, organization_uuid, first_name, last_name, email, phone_number, 
                 user_id, salutation, job_title, department, company_name, fax_number, 
                 website_url, gender, created_at, updated_at 
                 FROM module_crm_customers 
                 WHERE organization_uuid = ?1 
                 ORDER BY last_name ASC, first_name ASC 
                 LIMIT ?2 OFFSET ?3"
            )
            .bind(organization_uuid)
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(p)
            .await?;
            
            rows.into_iter()
                .map(|row| CrmCustomer {
                    uuid: row.get("uuid"),
                    organization_uuid: row.get("organization_uuid"),
                    first_name: row.get("first_name"),
                    last_name: row.get("last_name"),
                    email: row.get::<Option<String>, _>("email"),
                    phone_number: row.get::<Option<String>, _>("phone_number"),
                    user_id: row.get::<Option<String>, _>("user_id"),
                    salutation: row.get::<Option<String>, _>("salutation"),
                    job_title: row.get::<Option<String>, _>("job_title"),
                    department: row.get::<Option<String>, _>("department"),
                    company_name: row.get::<Option<String>, _>("company_name"),
                    fax_number: row.get::<Option<String>, _>("fax_number"),
                    website_url: row.get::<Option<String>, _>("website_url"),
                    gender: row.get::<Option<String>, _>("gender"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
                })
                .collect()
        }
    };
    
    Ok((customers, total_count))
}

/// Load all conversations for a customer from the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `customer_uuid` - UUID of the customer to load conversations for
///
/// # Returns
/// Returns a vector of `CrmCustomerConversation` sorted by creation date (newest first)
///
/// # Errors
/// Returns `CrmCustomerDatabaseError` if the database query fails
pub async fn load_customer_conversations(
    pool: &DatabasePool,
    customer_uuid: &str,
) -> Result<Vec<CrmCustomerConversation>, CrmCustomerDatabaseError> {
    match pool {
        DatabasePool::MySql(p) => {
            let rows = sqlx::query(
                "SELECT conversation_uuid, customer_uuid, message, source, channel_uuid, created_at 
                 FROM module_crm_customer_conversations 
                 WHERE customer_uuid = ? 
                 ORDER BY created_at DESC",
            )
            .bind(customer_uuid)
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .map(|row| CrmCustomerConversation {
                    uuid: row.get("conversation_uuid"),
                    customer_uuid: row.get("customer_uuid"),
                    message: row.get("message"),
                    source: row.get("source"),
                    channel_uuid: row.get("channel_uuid"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                })
                .collect())
        }
        DatabasePool::Postgres(p) => {
            let rows = sqlx::query(
                "SELECT conversation_uuid, customer_uuid, message, source, channel_uuid, created_at 
                 FROM module_crm_customer_conversations 
                 WHERE customer_uuid = $1 
                 ORDER BY created_at DESC",
            )
            .bind(customer_uuid)
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .map(|row| CrmCustomerConversation {
                    uuid: row.get("conversation_uuid"),
                    customer_uuid: row.get("customer_uuid"),
                    message: row.get("message"),
                    source: row.get("source"),
                    channel_uuid: row.get("channel_uuid"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                })
                .collect())
        }
        DatabasePool::Sqlite(p) => {
            let rows = sqlx::query(
                "SELECT conversation_uuid, customer_uuid, message, source, channel_uuid, created_at 
                 FROM module_crm_customer_conversations 
                 WHERE customer_uuid = ?1 
                 ORDER BY created_at DESC",
            )
            .bind(customer_uuid)
            .fetch_all(p)
            .await?;

            Ok(rows
                .into_iter()
                .map(|row| CrmCustomerConversation {
                    uuid: row.get("conversation_uuid"),
                    customer_uuid: row.get("customer_uuid"),
                    message: row.get("message"),
                    source: row.get("source"),
                    channel_uuid: row.get("channel_uuid"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                })
                .collect())
        }
    }
}

/// Create a new customer conversation in the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `customer_uuid` - UUID of the customer the conversation belongs to
/// * `request` - Conversation creation request
///
/// # Returns
/// Returns the UUID of the newly created conversation
///
/// # Errors
/// Returns `CrmCustomerDatabaseError` if the database operation fails
pub async fn create_customer_conversation(
    pool: &DatabasePool,
    customer_uuid: &str,
    request: CreateCrmCustomerConversationRequest,
) -> Result<String, CrmCustomerDatabaseError> {
    let conversation_uuid = uuid::Uuid::new_v4().to_string();
    let now = Utc::now();

    match pool {
        DatabasePool::MySql(p) => {
            sqlx::query(
                "INSERT INTO module_crm_customer_conversations 
                 (conversation_uuid, customer_uuid, message, source, channel_uuid, created_at) 
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(&conversation_uuid)
            .bind(customer_uuid)
            .bind(&request.message)
            .bind(&request.source)
            .bind(&request.channel_uuid)
            .bind(now)
            .execute(p)
            .await?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO module_crm_customer_conversations 
                 (conversation_uuid, customer_uuid, message, source, channel_uuid, created_at) 
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(&conversation_uuid)
            .bind(customer_uuid)
            .bind(&request.message)
            .bind(&request.source)
            .bind(&request.channel_uuid)
            .bind(now)
            .execute(p)
            .await?;
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO module_crm_customer_conversations 
                 (conversation_uuid, customer_uuid, message, source, channel_uuid, created_at) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .bind(&conversation_uuid)
            .bind(customer_uuid)
            .bind(&request.message)
            .bind(&request.source)
            .bind(&request.channel_uuid)
            .bind(now)
            .execute(p)
            .await?;
        }
    }

    Ok(conversation_uuid)
}

/// Update a customer in the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `customer_uuid` - UUID of the customer to update
/// * `request` - Update request with fields to update (only Some fields will be updated)
///
/// # Errors
/// Returns `CrmCustomerDatabaseError` if the database operation fails
pub async fn update_customer(
    pool: &DatabasePool,
    customer_uuid: &str,
    request: UpdateCrmCustomerRequest,
) -> Result<(), CrmCustomerDatabaseError> {
    let now = Utc::now();

    // Build dynamic UPDATE query based on which fields are provided
    let mut update_fields = Vec::new();

    if request.first_name.is_some() {
        update_fields.push("first_name = ?");
    }
    if request.last_name.is_some() {
        update_fields.push("last_name = ?");
    }
    if request.email.is_some() {
        update_fields.push("email = ?");
    }
    if request.phone_number.is_some() {
        update_fields.push("phone_number = ?");
    }
    if request.user_id.is_some() {
        update_fields.push("user_id = ?");
    }
    if request.salutation.is_some() {
        update_fields.push("salutation = ?");
    }
    if request.job_title.is_some() {
        update_fields.push("job_title = ?");
    }
    if request.department.is_some() {
        update_fields.push("department = ?");
    }
    if request.company_name.is_some() {
        update_fields.push("company_name = ?");
    }
    if request.fax_number.is_some() {
        update_fields.push("fax_number = ?");
    }
    if request.website_url.is_some() {
        update_fields.push("website_url = ?");
    }
    if request.gender.is_some() {
        update_fields.push("gender = ?");
    }

    // Always update updated_at
    update_fields.push("updated_at = ?");

    if update_fields.is_empty() {
        // No fields to update
        return Ok(());
    }

    let update_clause = update_fields.join(", ");
    
    match pool {
        DatabasePool::MySql(p) => {
            let query_str = format!(
                "UPDATE module_crm_customers SET {} WHERE uuid = ?",
                update_clause
            );
            let mut query = sqlx::query(&query_str);

            if let Some(ref v) = request.first_name {
                query = query.bind(v);
            }
            if let Some(ref v) = request.last_name {
                query = query.bind(v);
            }
            if let Some(ref v) = request.email {
                query = query.bind(v);
            }
            if let Some(ref v) = request.phone_number {
                query = query.bind(v);
            }
            if let Some(ref v) = request.user_id {
                query = query.bind(v);
            }
            if let Some(ref v) = request.salutation {
                query = query.bind(v);
            }
            if let Some(ref v) = request.job_title {
                query = query.bind(v);
            }
            if let Some(ref v) = request.department {
                query = query.bind(v);
            }
            if let Some(ref v) = request.company_name {
                query = query.bind(v);
            }
            if let Some(ref v) = request.fax_number {
                query = query.bind(v);
            }
            if let Some(ref v) = request.website_url {
                query = query.bind(v);
            }
            if let Some(ref v) = request.gender {
                query = query.bind(v);
            }
            query = query.bind(now);
            query = query.bind(customer_uuid);

            query.execute(p).await?;
        }
        DatabasePool::Postgres(p) => {
            let mut bind_index = 1;
            let mut update_fields_pg = Vec::new();

            if request.first_name.is_some() {
                update_fields_pg.push(format!("first_name = ${}", bind_index));
                bind_index += 1;
            }
            if request.last_name.is_some() {
                update_fields_pg.push(format!("last_name = ${}", bind_index));
                bind_index += 1;
            }
            if request.email.is_some() {
                update_fields_pg.push(format!("email = ${}", bind_index));
                bind_index += 1;
            }
            if request.phone_number.is_some() {
                update_fields_pg.push(format!("phone_number = ${}", bind_index));
                bind_index += 1;
            }
            if request.user_id.is_some() {
                update_fields_pg.push(format!("user_id = ${}", bind_index));
                bind_index += 1;
            }
            if request.salutation.is_some() {
                update_fields_pg.push(format!("salutation = ${}", bind_index));
                bind_index += 1;
            }
            if request.job_title.is_some() {
                update_fields_pg.push(format!("job_title = ${}", bind_index));
                bind_index += 1;
            }
            if request.department.is_some() {
                update_fields_pg.push(format!("department = ${}", bind_index));
                bind_index += 1;
            }
            if request.company_name.is_some() {
                update_fields_pg.push(format!("company_name = ${}", bind_index));
                bind_index += 1;
            }
            if request.fax_number.is_some() {
                update_fields_pg.push(format!("fax_number = ${}", bind_index));
                bind_index += 1;
            }
            if request.website_url.is_some() {
                update_fields_pg.push(format!("website_url = ${}", bind_index));
                bind_index += 1;
            }
            if request.gender.is_some() {
                update_fields_pg.push(format!("gender = ${}", bind_index));
                bind_index += 1;
            }
            update_fields_pg.push(format!("updated_at = ${}", bind_index));
            bind_index += 1;

            let update_clause_pg = update_fields_pg.join(", ");
            let query_str = format!(
                "UPDATE module_crm_customers SET {} WHERE uuid = ${}",
                update_clause_pg, bind_index
            );
            let mut query = sqlx::query(&query_str);

            if let Some(ref v) = request.first_name {
                query = query.bind(v);
            }
            if let Some(ref v) = request.last_name {
                query = query.bind(v);
            }
            if let Some(ref v) = request.email {
                query = query.bind(v);
            }
            if let Some(ref v) = request.phone_number {
                query = query.bind(v);
            }
            if let Some(ref v) = request.user_id {
                query = query.bind(v);
            }
            if let Some(ref v) = request.salutation {
                query = query.bind(v);
            }
            if let Some(ref v) = request.job_title {
                query = query.bind(v);
            }
            if let Some(ref v) = request.department {
                query = query.bind(v);
            }
            if let Some(ref v) = request.company_name {
                query = query.bind(v);
            }
            if let Some(ref v) = request.fax_number {
                query = query.bind(v);
            }
            if let Some(ref v) = request.website_url {
                query = query.bind(v);
            }
            if let Some(ref v) = request.gender {
                query = query.bind(v);
            }
            query = query.bind(now);
            query = query.bind(customer_uuid);

            query.execute(p).await?;
        }
        DatabasePool::Sqlite(p) => {
            let mut bind_index = 1;
            let mut update_fields_sqlite = Vec::new();

            if request.first_name.is_some() {
                update_fields_sqlite.push(format!("first_name = ?{}", bind_index));
                bind_index += 1;
            }
            if request.last_name.is_some() {
                update_fields_sqlite.push(format!("last_name = ?{}", bind_index));
                bind_index += 1;
            }
            if request.email.is_some() {
                update_fields_sqlite.push(format!("email = ?{}", bind_index));
                bind_index += 1;
            }
            if request.phone_number.is_some() {
                update_fields_sqlite.push(format!("phone_number = ?{}", bind_index));
                bind_index += 1;
            }
            if request.user_id.is_some() {
                update_fields_sqlite.push(format!("user_id = ?{}", bind_index));
                bind_index += 1;
            }
            if request.salutation.is_some() {
                update_fields_sqlite.push(format!("salutation = ?{}", bind_index));
                bind_index += 1;
            }
            if request.job_title.is_some() {
                update_fields_sqlite.push(format!("job_title = ?{}", bind_index));
                bind_index += 1;
            }
            if request.department.is_some() {
                update_fields_sqlite.push(format!("department = ?{}", bind_index));
                bind_index += 1;
            }
            if request.company_name.is_some() {
                update_fields_sqlite.push(format!("company_name = ?{}", bind_index));
                bind_index += 1;
            }
            if request.fax_number.is_some() {
                update_fields_sqlite.push(format!("fax_number = ?{}", bind_index));
                bind_index += 1;
            }
            if request.website_url.is_some() {
                update_fields_sqlite.push(format!("website_url = ?{}", bind_index));
                bind_index += 1;
            }
            if request.gender.is_some() {
                update_fields_sqlite.push(format!("gender = ?{}", bind_index));
                bind_index += 1;
            }
            update_fields_sqlite.push(format!("updated_at = ?{}", bind_index));
            bind_index += 1;

            let update_clause_sqlite = update_fields_sqlite.join(", ");
            let query_str = format!(
                "UPDATE module_crm_customers SET {} WHERE uuid = ?{}",
                update_clause_sqlite, bind_index
            );
            let mut query = sqlx::query(&query_str);

            if let Some(ref v) = request.first_name {
                query = query.bind(v);
            }
            if let Some(ref v) = request.last_name {
                query = query.bind(v);
            }
            if let Some(ref v) = request.email {
                query = query.bind(v);
            }
            if let Some(ref v) = request.phone_number {
                query = query.bind(v);
            }
            if let Some(ref v) = request.user_id {
                query = query.bind(v);
            }
            if let Some(ref v) = request.salutation {
                query = query.bind(v);
            }
            if let Some(ref v) = request.job_title {
                query = query.bind(v);
            }
            if let Some(ref v) = request.department {
                query = query.bind(v);
            }
            if let Some(ref v) = request.company_name {
                query = query.bind(v);
            }
            if let Some(ref v) = request.fax_number {
                query = query.bind(v);
            }
            if let Some(ref v) = request.website_url {
                query = query.bind(v);
            }
            if let Some(ref v) = request.gender {
                query = query.bind(v);
            }
            query = query.bind(now);
            query = query.bind(customer_uuid);

            query.execute(p).await?;
        }
    }

    Ok(())
}

