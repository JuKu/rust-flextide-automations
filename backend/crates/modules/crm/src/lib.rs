mod api;
mod customer;

use axum::{
    extract::Extension,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use chrono::{Datelike, Utc};
use serde::Serialize;
use serde_json::json;

pub use customer::{
    CrmCustomer, CrmCustomerAddress, CrmCustomerNote, CreateCrmCustomerAddressRequest,
    CreateCrmCustomerNoteRequest, CreateCrmCustomerRequest,
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
    Extension(_org_uuid): Extension<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // TODO: Fetch from database based on org_uuid
    // For now, return mocked data
    
    // Mock data
    let response = KpiResponse {
        total_sales_this_month: 0.0,
        orders_this_month: 0,
        orders_last_month: 0,
        win_rate_this_month: 0.0,
        avg_days_to_close: 0.0,
        total_users: 0,
        open_deals: 0.0,
    };
    
    Ok(Json(json!(response)))
}

async fn get_customers(
    Extension(_org_uuid): Extension<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // TODO: Fetch from database based on org_uuid
    // For now, return empty list
    
    let response = CustomersResponse {
        customers: vec![],
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

