//! CRM Customer module
//!
//! Provides functionality for managing CRM customers, including database operations.

mod database;

pub use database::CrmCustomerDatabaseError;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// CRM Customer data structure
///
/// Represents a customer in the CRM system with all fields matching the database schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrmCustomer {
    pub uuid: String,
    pub organization_uuid: String,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub user_id: Option<String>,
    pub salutation: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub company_name: Option<String>,
    pub fax_number: Option<String>,
    pub website_url: Option<String>,
    pub gender: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request structure for creating a new customer
#[derive(Debug, Deserialize)]
pub struct CreateCrmCustomerRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub user_id: Option<String>,
    pub salutation: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub company_name: Option<String>,
    pub fax_number: Option<String>,
    pub website_url: Option<String>,
    pub gender: Option<String>,
}

/// CRM Customer Note data structure
///
/// Represents a note attached to a customer in the CRM system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrmCustomerNote {
    pub uuid: String,
    pub customer_uuid: String,
    pub note_text: String,
    pub author_id: String,
    pub visible_to_customer: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request structure for creating a new customer note
#[derive(Debug, Deserialize)]
pub struct CreateCrmCustomerNoteRequest {
    pub note_text: String,
    pub visible_to_customer: Option<bool>,
}

/// CRM Customer Address data structure
///
/// Represents an address attached to a customer in the CRM system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrmCustomerAddress {
    pub uuid: String,
    pub customer_uuid: String,
    pub address_type: String,
    pub street: Option<String>,
    pub city: Option<String>,
    pub state_province: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request structure for creating a new customer address
#[derive(Debug, Deserialize)]
pub struct CreateCrmCustomerAddressRequest {
    pub address_type: String,
    pub street: Option<String>,
    pub city: Option<String>,
    pub state_province: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub is_primary: Option<bool>,
}

impl CrmCustomer {
    /// Load a customer from the database by UUID
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `customer_uuid` - UUID of the customer to load
    ///
    /// # Errors
    /// Returns `CrmCustomerDatabaseError` if the database query fails or customer is not found
    pub async fn load_from_database(
        pool: &flextide_core::database::DatabasePool,
        customer_uuid: &str,
    ) -> Result<Self, CrmCustomerDatabaseError> {
        database::load_customer_by_uuid(pool, customer_uuid).await
    }

    /// Create a new customer in the database for the specified organization
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `organization_uuid` - UUID of the organization the customer belongs to
    /// * `request` - Customer creation request with customer data
    ///
    /// # Returns
    /// Returns the UUID of the newly created customer
    ///
    /// # Errors
    /// Returns `CrmCustomerDatabaseError` if the database operation fails
    pub async fn create_customer(
        pool: &flextide_core::database::DatabasePool,
        organization_uuid: &str,
        request: CreateCrmCustomerRequest,
    ) -> Result<String, CrmCustomerDatabaseError> {
        database::create_customer(pool, organization_uuid, request).await
    }

    /// Add a new note to this customer
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `author_id` - UUID of the user creating the note (must not be empty)
    /// * `request` - Note creation request with note data
    ///
    /// # Returns
    /// Returns the UUID of the newly created note
    ///
    /// # Errors
    /// Returns `CrmCustomerDatabaseError` if validation fails or the database operation fails
    ///
    /// # Validation
    /// - `author_id` must not be empty
    /// - `note_text` must not be empty and must have at least 2 characters
    /// - `visible_to_customer` defaults to `false` if not specified
    pub async fn add_note(
        &self,
        pool: &flextide_core::database::DatabasePool,
        author_id: &str,
        request: CreateCrmCustomerNoteRequest,
    ) -> Result<String, CrmCustomerDatabaseError> {
        // Validate author_id
        if author_id.trim().is_empty() {
            return Err(CrmCustomerDatabaseError::EmptyAuthorId);
        }

        // Validate note_text
        if request.note_text.trim().is_empty() || request.note_text.trim().len() < 2 {
            return Err(CrmCustomerDatabaseError::InvalidNoteText);
        }

        database::create_customer_note(pool, &self.uuid, author_id, request).await
    }

    /// Delete a note from this customer
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `note_uuid` - UUID of the note to delete
    ///
    /// # Returns
    /// Returns `Ok(())` if the note was successfully deleted
    ///
    /// # Errors
    /// Returns `CrmCustomerDatabaseError` if:
    /// - The note does not exist
    /// - The note does not belong to this customer
    /// - The database operation fails
    pub async fn delete_note(
        &self,
        pool: &flextide_core::database::DatabasePool,
        note_uuid: &str,
    ) -> Result<(), CrmCustomerDatabaseError> {
        database::delete_customer_note(pool, &self.uuid, note_uuid).await
    }

    /// List all notes for this customer
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// Returns a vector of `CrmCustomerNote` sorted by creation date (oldest first)
    ///
    /// # Errors
    /// Returns `CrmCustomerDatabaseError` if the database query fails
    pub async fn list_notes(
        &self,
        pool: &flextide_core::database::DatabasePool,
    ) -> Result<Vec<CrmCustomerNote>, CrmCustomerDatabaseError> {
        database::load_customer_notes(pool, &self.uuid).await
    }

    /// Add a new address to this customer
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `request` - Address creation request with address data
    ///
    /// # Returns
    /// Returns the UUID of the newly created address
    ///
    /// # Errors
    /// Returns `CrmCustomerDatabaseError` if validation fails or the database operation fails
    ///
    /// # Validation
    /// - `address_type` must not be empty
    /// - `is_primary` defaults to `false` if not specified
    pub async fn add_address(
        &self,
        pool: &flextide_core::database::DatabasePool,
        request: CreateCrmCustomerAddressRequest,
    ) -> Result<String, CrmCustomerDatabaseError> {
        // Validate address_type
        if request.address_type.trim().is_empty() {
            return Err(CrmCustomerDatabaseError::EmptyAddressType);
        }

        database::create_customer_address(pool, &self.uuid, request).await
    }

    /// Delete an address from this customer
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `address_uuid` - UUID of the address to delete
    ///
    /// # Returns
    /// Returns `Ok(())` if the address was successfully deleted
    ///
    /// # Errors
    /// Returns `CrmCustomerDatabaseError` if:
    /// - The address does not exist
    /// - The address does not belong to this customer
    /// - The database operation fails
    pub async fn delete_address(
        &self,
        pool: &flextide_core::database::DatabasePool,
        address_uuid: &str,
    ) -> Result<(), CrmCustomerDatabaseError> {
        database::delete_customer_address(pool, &self.uuid, address_uuid).await
    }

    /// Delete this customer from the database
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// Returns `Ok(())` if the customer was successfully deleted
    ///
    /// # Errors
    /// Returns `CrmCustomerDatabaseError` if:
    /// - The customer does not exist
    /// - The database operation fails
    ///
    /// # Note
    /// This will cascade delete all related records (notes, addresses, conversations) due to foreign key constraints
    pub async fn delete(
        self,
        pool: &flextide_core::database::DatabasePool,
    ) -> Result<(), CrmCustomerDatabaseError> {
        database::delete_customer(pool, &self.uuid).await
    }
}

