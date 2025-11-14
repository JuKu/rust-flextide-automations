//! CRM API endpoints
//!
//! Provides REST API endpoints for managing CRM customers, notes, and addresses.

use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get, post},
    Router,
};
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};

use crate::customer::{
    CreateCrmCustomerAddressRequest, CreateCrmCustomerConversationRequest,
    CreateCrmCustomerNoteRequest, CreateCrmCustomerRequest, CrmCustomer, UpdateCrmCustomerRequest,
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
    tracing::debug!(
        "[CRM create_customer] Checking organization membership: user_uuid={}, org_uuid={}",
        claims.user_uuid,
        org_uuid
    );
    
    let belongs = user_belongs_to_organization(&pool, &claims.user_uuid, &org_uuid)
        .await
        .map_err(|e| {
            tracing::error!(
                "[CRM create_customer] Database error checking organization membership: user_uuid={}, org_uuid={}, error={}",
                claims.user_uuid,
                org_uuid,
                e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    tracing::debug!(
        "[CRM create_customer] Organization membership check result: user_uuid={}, org_uuid={}, belongs={}",
        claims.user_uuid,
        org_uuid,
        belongs
    );

    if !belongs {
        tracing::warn!(
            "[CRM create_customer] User {} does not belong to organization {}",
            claims.user_uuid,
            org_uuid
        );
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
/// DELETE /api/modules/crm/customers/{uuid}
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
/// POST /api/modules/crm/customers/{uuid}/notes
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
/// DELETE /api/modules/crm/customers/{uuid}/notes/{note_uuid}
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
/// POST /api/modules/crm/customers/{uuid}/addresses
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
/// DELETE /api/modules/crm/customers/{uuid}/addresses/{address_uuid}
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

/// Query parameters for customer search
#[derive(Debug, Deserialize)]
pub struct SearchCustomersQuery {
    pub q: String,
}

/// Search customers
///
/// GET /api/modules/crm/customers/search?q=<query>
pub async fn search_customers(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<SearchCustomersQuery>,
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
    let has_permission = user_has_permission(&pool, &claims.user_uuid, &org_uuid, "module_crm_search_customers")
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
            Json(json!({ "error": "User does not have permission to search customers" })),
        ));
    }

    // Validate query
    if params.q.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Search query cannot be empty" })),
        ));
    }

    // Search customers
    let customers = CrmCustomer::search_customers(&pool, &org_uuid, &params.q)
        .await
        .map_err(|e| {
            tracing::error!("Error searching customers: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to search customers" })),
            )
        })?;

    // Convert to response format (matching the frontend Customer type)
    let customer_responses: Vec<serde_json::Value> = customers
        .into_iter()
        .map(|c| {
            json!({
                "id": c.uuid,
                "name": format!("{} {}", c.first_name, c.last_name),
                "email": c.email.unwrap_or_default(),
                "company": c.company_name,
                "status": "", // TODO: Add status field if needed
                "created_at": c.created_at.to_rfc3339(),
                "last_contact": None::<String>, // TODO: Add last_contact if needed
            })
        })
        .collect();

    Ok(Json(json!({
        "customers": customer_responses
    })))
}

/// Get a single customer by UUID
///
/// GET /api/modules/crm/customers/{uuid}
pub async fn get_customer(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Path(customer_uuid): Path<String>,
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
    let has_permission = user_has_permission(&pool, &claims.user_uuid, &org_uuid, "module_crm_can_see_customer")
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
            Json(json!({ "error": "User does not have permission to view customer details" })),
        ));
    }

    // Load customer
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

    Ok(Json(json!(customer)))
}

/// Get customer KPIs
///
/// GET /api/modules/crm/customers/{uuid}/kpis
pub async fn get_customer_kpis(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Path(customer_uuid): Path<String>,
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
    let has_permission = user_has_permission(&pool, &claims.user_uuid, &org_uuid, "module_crm_can_see_customer")
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
            Json(json!({ "error": "User does not have permission to view customer details" })),
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

    // TODO: Fetch real KPIs from database
    // For now, return mocked data
    let kpis = json!({
        "clv": 12500.50, // Customer Lifetime Value in €
        "avg_deal_amount": 2500.00, // Average money amount per deal
        "org_avg_deal_amount": 2200.00, // Average of all customers in organization
        "last_deal_date": Option::<String>::None, // Last deal date
        "current_sale_status": "Has obtained a quote", // Pipeline status
        "source": "Website", // Where customer came from
        "assigned_user": customer.user_id, // Assigned user UUID
        "days_since_last_contact": 5, // Days since last contact
        "last_interaction_date": Option::<String>::None, // Last interaction date
        "created_at": customer.created_at.to_rfc3339(),
    });

    Ok(Json(kpis))
}

/// Get customer notes
///
/// GET /api/modules/crm/customers/{uuid}/notes
pub async fn get_customer_notes(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Path(customer_uuid): Path<String>,
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
    let has_permission = user_has_permission(&pool, &claims.user_uuid, &org_uuid, "module_crm_can_see_customer")
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
            Json(json!({ "error": "User does not have permission to view customer details" })),
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

    // Load notes
    let notes = customer.list_notes(&pool).await.map_err(|e| {
        tracing::error!("Error loading notes: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to load notes" })),
        )
    })?;

    Ok(Json(json!(notes)))
}

/// Get customer conversations
///
/// GET /api/modules/crm/customers/{uuid}/conversations
pub async fn get_customer_conversations(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Path(customer_uuid): Path<String>,
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
    let has_permission = user_has_permission(&pool, &claims.user_uuid, &org_uuid, "module_crm_can_see_customer")
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
            Json(json!({ "error": "User does not have permission to view customer details" })),
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

    // Load conversations
    let conversations = customer.list_conversations(&pool).await.map_err(|e| {
        tracing::error!("Error loading conversations: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to load conversations" })),
        )
    })?;

    Ok(Json(json!(conversations)))
}

/// Add a conversation to a customer
///
/// POST /api/modules/crm/customers/{uuid}/conversations
pub async fn add_customer_conversation(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Path(customer_uuid): Path<String>,
    Json(request): Json<CreateCrmCustomerConversationRequest>,
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
            Json(json!({ "error": "User does not have permission to add customer conversations" })),
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

    // Add conversation
    let conversation_uuid = customer
        .add_conversation(&pool, request)
        .await
        .map_err(|e| {
            tracing::error!("Error adding conversation: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to add conversation" })),
            )
        })?;

    Ok(Json(json!({
        "uuid": conversation_uuid,
        "message": "Conversation added successfully"
    })))
}

/// Update a customer
///
/// PUT /api/modules/crm/customers/{uuid}
pub async fn update_customer(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Path(customer_uuid): Path<String>,
    Json(request): Json<UpdateCrmCustomerRequest>,
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
    let has_permission = user_has_permission(&pool, &claims.user_uuid, &org_uuid, "module_crm_can_edit_customers")
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
            Json(json!({ "error": "User does not have permission to edit customers" })),
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

    // Update customer
    customer
        .update(&pool, request)
        .await
        .map_err(|e| {
            tracing::error!("Error updating customer: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to update customer" })),
            )
        })?;

    Ok(Json(json!({
        "message": "Customer updated successfully"
    })))
}

/// Create the API router for CRM endpoints
pub fn create_api_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/modules/crm/customers", post(create_customer))
        .route("/modules/crm/customers/search", get(search_customers))
        .route("/modules/crm/customers/{uuid}", get(get_customer).put(update_customer).delete(delete_customer))
        .route("/modules/crm/customers/{uuid}/kpis", get(get_customer_kpis))
        .route("/modules/crm/customers/{uuid}/notes", get(get_customer_notes).post(add_customer_note))
        .route(
            "/modules/crm/customers/{uuid}/notes/{note_uuid}",
            delete(delete_customer_note),
        )
        .route("/modules/crm/customers/{uuid}/conversations", get(get_customer_conversations).post(add_customer_conversation))
        .route("/modules/crm/customers/{uuid}/addresses", post(add_customer_address))
        .route(
            "/modules/crm/customers/{uuid}/addresses/{address_uuid}",
            delete(delete_customer_address),
        )
}

