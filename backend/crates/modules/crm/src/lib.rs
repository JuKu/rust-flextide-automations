mod api;
mod customer;

use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use chrono::{Datelike, Utc};
use flextide_core::database::DatabasePool;
use flextide_core::jwt::Claims;
use flextide_core::user::{user_belongs_to_organization, user_has_permission};
use serde::Serialize;
use serde_json::json;
use sqlx::Row;

pub use customer::{
    CrmCustomer, CrmCustomerAddress, CrmCustomerConversation, CrmCustomerNote,
    CreateCrmCustomerAddressRequest, CreateCrmCustomerConversationRequest,
    CreateCrmCustomerNoteRequest, CreateCrmCustomerRequest, UpdateCrmCustomerRequest,
};

pub fn create_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/modules/crm/kpis", get(get_kpis))
        .route("/modules/crm/customers", get(get_customers))
        .route("/modules/crm/sales-pipeline-chart", get(get_sales_pipeline_chart))
        .route("/modules/crm/countries-chart", get(get_countries_chart))
        .route("/modules/crm/closed-deals", get(get_closed_deals))
        .merge(api::create_api_router())
}

#[derive(Debug, Serialize)]
pub struct KpiResponse {
    pub total_sales_this_month: f64,
    pub orders_this_month: u32,
    pub orders_last_month: u32,
    pub win_rate_this_month: f64,
    pub avg_days_to_close: f64,
    pub total_users: u32,
    pub open_deals: f64,
}

#[derive(Debug, Serialize)]
pub struct Customer {
    pub id: String,
    pub name: String,
    pub email: String,
    pub company: Option<String>,
    pub status: String,
    pub created_at: String,
    pub last_contact: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CustomersResponse {
    pub customers: Vec<Customer>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

#[derive(Debug, Serialize)]
pub struct PipelineStatus {
    pub status: String,
    pub count: u32,
}

#[derive(Debug, Serialize)]
pub struct SalesPipelineChartResponse {
    pub statuses: Vec<PipelineStatus>,
}

#[derive(Debug, Serialize)]
pub struct CountryData {
    pub country: String,
    pub count: u32,
}

#[derive(Debug, Serialize)]
pub struct CountriesChartResponse {
    pub countries: Vec<CountryData>,
}

#[derive(Debug, Serialize)]
pub struct ClosedDealData {
    pub month: String,
    pub current_year: f64,
    pub previous_year: f64,
}

#[derive(Debug, Serialize)]
pub struct ClosedDealsResponse {
    pub deals: Vec<ClosedDealData>,
}

async fn get_kpis(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Count total customers for the organization
    let total_customers = match &pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM module_crm_customers WHERE organization_uuid = ?")
                .bind(&org_uuid)
                .fetch_one(p)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to count customers for organization {}: {}", org_uuid, e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Failed to fetch customer count" })),
                    )
                })?;
            let count: i64 = row.get("count");
            count as u32
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM module_crm_customers WHERE organization_uuid = $1")
                .bind(&org_uuid)
                .fetch_one(p)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to count customers for organization {}: {}", org_uuid, e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Failed to fetch customer count" })),
                    )
                })?;
            let count: i64 = row.get("count");
            count as u32
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM module_crm_customers WHERE organization_uuid = ?1")
                .bind(&org_uuid)
                .fetch_one(p)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to count customers for organization {}: {}", org_uuid, e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Failed to fetch customer count" })),
                    )
                })?;
            let count: i64 = row.get("count");
            count as u32
        }
    };
    
    // TODO: Fetch other KPIs from database based on org_uuid
    // For now, return mocked data for other fields
    let response = KpiResponse {
        total_sales_this_month: 0.0,
        orders_this_month: 0,
        orders_last_month: 0,
        win_rate_this_month: 0.0,
        avg_days_to_close: 0.0,
        total_users: total_customers,
        open_deals: 0.0,
    };
    
    Ok(Json(json!(response)))
}

#[derive(Debug, serde::Deserialize)]
struct CustomersQuery {
    page: Option<u32>,
    page_size: Option<u32>,
}

async fn get_customers(
    Extension(pool): Extension<DatabasePool>,
    Extension(org_uuid): Extension<String>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<CustomersQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
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
    let has_permission = user_has_permission(&pool, &claims.user_uuid, &org_uuid, "module_crm_can_see_all_customers")
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
            Json(json!({ "error": "User does not have permission to view all customers" })),
        ));
    }

    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(50).min(50);
    
    // Fetch customers with pagination
    let (crm_customers, total_count) = CrmCustomer::list_customers_paginated(&pool, &org_uuid, page, page_size)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list customers: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to fetch customers" })),
            )
        })?;
    
    // Convert to response format
    let customers: Vec<Customer> = crm_customers
        .into_iter()
        .map(|c| Customer {
            id: c.uuid,
            name: format!("{} {}", c.first_name, c.last_name),
            email: c.email.unwrap_or_default(),
            company: c.company_name,
            status: "Active".to_string(), // TODO: Add status field to database
            created_at: c.created_at.to_rfc3339(),
            last_contact: None, // TODO: Add last_contact field to database
        })
        .collect();
    
    let total_pages = (total_count as f64 / page_size as f64).ceil() as u32;
    
    let response = CustomersResponse {
        customers,
        total: total_count,
        page,
        page_size,
        total_pages,
    };
    
    Ok(Json(json!(response)))
}

async fn get_sales_pipeline_chart(
    Extension(_org_uuid): Extension<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // TODO: Fetch from database based on org_uuid
    // For now, return mocked data with configurable pipeline statuses
    
    let response = SalesPipelineChartResponse {
        statuses: vec![
            PipelineStatus {
                status: "Was interested in the product".to_string(),
                count: 15,
            },
            PipelineStatus {
                status: "Has obtained a quote".to_string(),
                count: 12,
            },
            PipelineStatus {
                status: "Inquired about the offer".to_string(),
                count: 8,
            },
            PipelineStatus {
                status: "Has change requests".to_string(),
                count: 5,
            },
            PipelineStatus {
                status: "Accepted the contract".to_string(),
                count: 3,
            },
            PipelineStatus {
                status: "Payed the money".to_string(),
                count: 2,
            },
            PipelineStatus {
                status: "Completed".to_string(),
                count: 1,
            },
        ],
    };
    
    Ok(Json(json!(response)))
}

async fn get_countries_chart(
    Extension(_org_uuid): Extension<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // TODO: Fetch from database based on org_uuid
    // For now, return mocked data
    
    let response = CountriesChartResponse {
        countries: vec![
            CountryData {
                country: "Germany".to_string(),
                count: 25,
            },
            CountryData {
                country: "United States".to_string(),
                count: 18,
            },
            CountryData {
                country: "United Kingdom".to_string(),
                count: 12,
            },
            CountryData {
                country: "France".to_string(),
                count: 8,
            },
            CountryData {
                country: "Spain".to_string(),
                count: 5,
            },
            CountryData {
                country: "Other".to_string(),
                count: 10,
            },
        ],
    };
    
    Ok(Json(json!(response)))
}

async fn get_closed_deals(
    Extension(_org_uuid): Extension<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // TODO: Fetch from database based on org_uuid
    // For now, return mocked data for last 12 months
    
    let now = Utc::now();
    let mut deals = Vec::new();
    
    // Generate data for last 12 months
    for i in 0..12 {
        let month_date = now - chrono::Duration::days(30 * (11 - i) as i64);
        let month_name = format!("{:02}/{}", month_date.month(), month_date.year() % 100);
        
        // Mock data - current year values (using deterministic pattern)
        let current_year_value = (50.0 + (i as f64 * 10.0) + ((i as f64 * 7.0) % 20.0)).round();
        // Mock data - previous year values (slightly lower)
        let previous_year_value = (current_year_value * 0.85).round();
        
        deals.push(ClosedDealData {
            month: month_name,
            current_year: current_year_value,
            previous_year: previous_year_value,
        });
    }
    
    let response = ClosedDealsResponse { deals };
    
    Ok(Json(json!(response)))
}

