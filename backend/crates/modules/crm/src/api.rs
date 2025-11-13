//! CRM API endpoints
//!
//! Provides REST API endpoints for managing CRM customers, notes, and addresses.

use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, post},
    Router,
};
use serde_json::{json, Value as JsonValue};

use crate::customer::{
    CreateCrmCustomerAddressRequest, CreateCrmCustomerNoteRequest, CreateCrmCustomerRequest,
    CrmCustomer,
};
use flextide_core::database::DatabasePool;
use flextide_core::jwt::Claims;
use flextide_core::user::{user_belongs_to_organization, user_has_permission};

/// Create a new customer
///
/// POST /api/modules/crm/customers
pub async fn create_customer(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreateCrmCustomerRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // Sanity checks
    if request.first_name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "First name cannot be empty" })),
        ));
    }

    if request.last_name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Last name cannot be empty" })),
        ));
    }

    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(&pool, &claims.user_uuid, &org_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    if !belongs {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "User does not belong to this organization" })),
        ));
    }

    // Check permission
    let has_permission = user_has_permission(&pool, &claims.user_uuid, &org_uuid, "module_crm_can_create_customers")
        .await
        .map_err(|e| {
            tracing::error!("Database error checking permission: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    if !has_permission {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "User does not have permission to create customers" })),
        ));
    }

    // Create customer
    let customer_uuid = CrmCustomer::create_customer(&pool, &org_uuid, request)
        .await
        .map_err(|e| {
            tracing::error!("Error creating customer: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to create customer" })),
            )
        })?;

    Ok(Json(json!({
        "uuid": customer_uuid,
        "message": "Customer created successfully"
    })))
}

/// Delete a customer by UUID
///
/// DELETE /api/modules/crm/customers/:uuid
pub async fn delete_customer(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Path(customer_uuid): Path<String>,
) -> Result<Json<JsonValue>, (StatusCode, Json<JsonValue>)> {
    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(&pool, &claims.user_uuid, &org_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    if !belongs {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "User does not belong to this organization" })),
        ));
    }

    // Check permission
    let has_permission = user_has_permission(&pool, &claims.user_uuid, &org_uuid, "module_crm_can_delete_customers")
        .await
        .map_err(|e| {
            tracing::error!("Database error checking permission: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    if !has_permission {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "User does not have permission to delete customers" })),
        ));
    }

    // Load customer to verify it belongs to the organization
    let customer = CrmCustomer::load_from_database(&pool, &customer_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Error loading customer: {}", e);
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Customer not found" })),
            )
        })?;

    // Verify customer belongs to the organization
    if customer.organization_uuid != org_uuid {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Customer does not belong to this organization" })),
        ));
    }

    // Delete customer
    customer
        .delete(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Error deleting customer: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to delete customer" })),
            )
        })?;

    Ok(Json(json!({
        "message": "Customer deleted successfully"
    })))
}

/// Add a note to a customer
///
/// POST /api/modules/crm/customers/:uuid/notes
pub async fn add_customer_note(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Path(customer_uuid): Path<String>,
    Json(request): Json<CreateCrmCustomerNoteRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(&pool, &claims.user_uuid, &org_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    if !belongs {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "User does not belong to this organization" })),
        ));
    }

    // Check permission
    let has_permission = user_has_permission(&pool, &claims.user_uuid, &org_uuid, "module_crm_can_add_customer_notes")
        .await
        .map_err(|e| {
            tracing::error!("Database error checking permission: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    if !has_permission {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "User does not have permission to add customer notes" })),
        ));
    }

    // Load customer to verify it belongs to the organization
    let customer = CrmCustomer::load_from_database(&pool, &customer_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Error loading customer: {}", e);
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Customer not found" })),
            )
        })?;

    // Verify customer belongs to the organization
    if customer.organization_uuid != org_uuid {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Customer does not belong to this organization" })),
        ));
    }

    // Add note
    let note_uuid = customer
        .add_note(&pool, &claims.user_uuid, request)
        .await
        .map_err(|e| {
            tracing::error!("Error adding note: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to add note" })),
            )
        })?;

    Ok(Json(json!({
        "uuid": note_uuid,
        "message": "Note added successfully"
    })))
}

/// Delete a note from a customer
///
/// DELETE /api/modules/crm/customers/:uuid/notes/:note_uuid
pub async fn delete_customer_note(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Path((customer_uuid, note_uuid)): Path<(String, String)>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(&pool, &claims.user_uuid, &org_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    if !belongs {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "User does not belong to this organization" })),
        ));
    }

    // Check permission
    let has_permission = user_has_permission(&pool, &claims.user_uuid, &org_uuid, "module_crm_can_delete_customer_notes")
        .await
        .map_err(|e| {
            tracing::error!("Database error checking permission: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    if !has_permission {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "User does not have permission to delete customer notes" })),
        ));
    }

    // Load customer to verify it belongs to the organization
    let customer = CrmCustomer::load_from_database(&pool, &customer_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Error loading customer: {}", e);
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Customer not found" })),
            )
        })?;

    // Verify customer belongs to the organization
    if customer.organization_uuid != org_uuid {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Customer does not belong to this organization" })),
        ));
    }

    // Delete note
    customer
        .delete_note(&pool, &note_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Error deleting note: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to delete note" })),
            )
        })?;

    Ok(Json(json!({
        "message": "Note deleted successfully"
    })))
}

/// Add an address to a customer
///
/// POST /api/modules/crm/customers/:uuid/addresses
pub async fn add_customer_address(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Path(customer_uuid): Path<String>,
    Json(request): Json<CreateCrmCustomerAddressRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(&pool, &claims.user_uuid, &org_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    if !belongs {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "User does not belong to this organization" })),
        ));
    }

    // Check permission
    let has_permission = user_has_permission(&pool, &claims.user_uuid, &org_uuid, "module_crm_can_add_customer_addresses")
        .await
        .map_err(|e| {
            tracing::error!("Database error checking permission: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    if !has_permission {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "User does not have permission to add customer addresses" })),
        ));
    }

    // Load customer to verify it belongs to the organization
    let customer = CrmCustomer::load_from_database(&pool, &customer_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Error loading customer: {}", e);
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Customer not found" })),
            )
        })?;

    // Verify customer belongs to the organization
    if customer.organization_uuid != org_uuid {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Customer does not belong to this organization" })),
        ));
    }

    // Add address
    let address_uuid = customer
        .add_address(&pool, request)
        .await
        .map_err(|e| {
            tracing::error!("Error adding address: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to add address" })),
            )
        })?;

    Ok(Json(json!({
        "uuid": address_uuid,
        "message": "Address added successfully"
    })))
}

/// Delete an address from a customer
///
/// DELETE /api/modules/crm/customers/:uuid/addresses/:address_uuid
pub async fn delete_customer_address(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Path((customer_uuid, address_uuid)): Path<(String, String)>,
) -> Result<impl IntoResponse, (StatusCode, Json<JsonValue>)> {
    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(&pool, &claims.user_uuid, &org_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking organization membership: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    if !belongs {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "User does not belong to this organization" })),
        ));
    }

    // Check permission
    let has_permission = user_has_permission(&pool, &claims.user_uuid, &org_uuid, "module_crm_can_delete_customer_addresses")
        .await
        .map_err(|e| {
            tracing::error!("Database error checking permission: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    if !has_permission {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "User does not have permission to delete customer addresses" })),
        ));
    }

    // Load customer to verify it belongs to the organization
    let customer = CrmCustomer::load_from_database(&pool, &customer_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Error loading customer: {}", e);
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Customer not found" })),
            )
        })?;

    // Verify customer belongs to the organization
    if customer.organization_uuid != org_uuid {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Customer does not belong to this organization" })),
        ));
    }

    // Delete address
    customer
        .delete_address(&pool, &address_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Error deleting address: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to delete address" })),
            )
        })?;

    Ok(Json(json!({
        "message": "Address deleted successfully"
    })))
}

/// Create the API router for CRM endpoints
pub fn create_api_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/modules/crm/customers", post(create_customer))
        .route("/modules/crm/customers/:uuid", delete(delete_customer))
        .route("/modules/crm/customers/:uuid/notes", post(add_customer_note))
        .route(
            "/modules/crm/customers/:uuid/notes/:note_uuid",
            delete(delete_customer_note),
        )
        .route("/modules/crm/customers/:uuid/addresses", post(add_customer_address))
        .route(
            "/modules/crm/customers/:uuid/addresses/:address_uuid",
            delete(delete_customer_address),
        )
}

