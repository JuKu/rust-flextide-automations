use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

#[derive(Clone)]
pub struct AppState {
    pub jwt_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // email
    pub user_uuid: String, // user UUID
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    #[allow(dead_code)] // Will be used when implementing proper registration
    pub password: String,
}

/// Create the API router with all routes
pub fn create_app(state: AppState) -> Router {
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Request logging layer
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &axum::http::Request<_>| {
            tracing::info_span!(
                "http_request",
                method = %request.method(),
                uri = %request.uri(),
                version = ?request.version(),
            )
        })
        .on_request(|request: &axum::http::Request<_>, _span: &tracing::Span| {
            tracing::info!(
                method = %request.method(),
                uri = %request.uri(),
                "Incoming request"
            );
        })
        .on_response(|response: &axum::http::Response<_>, latency: std::time::Duration, _span: &tracing::Span| {
            let status = response.status();
            if status.is_client_error() || status.is_server_error() {
                tracing::warn!(
                    status = %status,
                    latency_ms = latency.as_millis(),
                    "Request completed"
                );
            } else {
                tracing::info!(
                    status = %status,
                    latency_ms = latency.as_millis(),
                    "Request completed"
                );
            }
        })
        .on_failure(|error: tower_http::classify::ServerErrorsFailureClass, latency: std::time::Duration, _span: &tracing::Span| {
            tracing::error!(
                error = ?error,
                latency_ms = latency.as_millis(),
                "Request failed"
            );
        });

    // Build router
    Router::new()
        .route("/api/health", get(health_check))
        .route("/api/login", post(login))
        .route("/api/register", post(register))
        .route("/api/logout", post(logout))
        .route("/api/organizations/list-own", get(list_own_organizations))
        .route("/api/workflows/{workflow_uuid}/edit-title", post(edit_workflow_title))
        .layer(
            ServiceBuilder::new()
                .layer(trace_layer)
                .layer(cors)
        )
        .with_state(state)
}

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Temporary: accept only admin@example.com / admin
    if payload.email != "admin@example.com" || payload.password != "admin" {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid email or password" })),
        ));
    }

    // Generate JWT token
    let now = Utc::now();
    let exp = (now + Duration::hours(24)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    // Generate a UUID for the user (in production, get from database)
    // For now, use a deterministic UUID based on email hash
    let user_uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, payload.email.as_bytes()).to_string();

    let claims = Claims {
        sub: payload.email.clone(),
        user_uuid: user_uuid.clone(),
        exp,
        iat,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_ref()),
    )
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to generate token" })),
        )
    })?;

    Ok(Json(json!({
        "token": token,
        "email": payload.email
    })))
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Temporary: accept any registration, but for now just return success
    // In production, you would:
    // 1. Validate email format
    // 2. Hash password
    // 3. Store in database
    // 4. Return appropriate response

    // Generate JWT token
    let now = Utc::now();
    let exp = (now + Duration::hours(24)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    // Generate a UUID for the user (in production, get from database)
    // For now, use a deterministic UUID based on email hash
    let user_uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, payload.email.as_bytes()).to_string();

    let claims = Claims {
        sub: payload.email.clone(),
        user_uuid: user_uuid.clone(),
        exp,
        iat,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_ref()),
    )
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to generate token" })),
        )
    })?;

    Ok(Json(json!({
        "token": token,
        "email": payload.email
    })))
}

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub user_uuid: String,
}

pub async fn logout(
    State(_state): State<AppState>,
    Json(payload): Json<LogoutRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    tracing::info!("User with userUUID {} has logged out", payload.user_uuid);
    
    Ok(Json(json!({ "message": "Logged out successfully" })))
}

#[derive(Debug, Serialize)]
pub struct Organization {
    pub uuid: String,
    pub title: String,
    pub is_admin: bool,
}

pub async fn list_own_organizations(
    State(_state): State<AppState>,
) -> Result<Json<Vec<Organization>>, (StatusCode, Json<Value>)> {
    // Mock data - in production, fetch from database
    let organizations = vec![
        Organization {
            uuid: uuid::Uuid::new_v4().to_string(),
            title: "My Organization".to_string(),
            is_admin: true,
        },
        Organization {
            uuid: uuid::Uuid::new_v4().to_string(),
            title: "Another Org".to_string(),
            is_admin: true,
        },
        Organization {
            uuid: uuid::Uuid::new_v4().to_string(),
            title: "Test Org".to_string(),
            is_admin: true,
        },
        Organization {
            uuid: uuid::Uuid::new_v4().to_string(),
            title: "Test Org 2".to_string(),
            is_admin: false,
        },
        Organization {
            uuid: uuid::Uuid::new_v4().to_string(),
            title: "Test Org 3".to_string(),
            is_admin: false,
        },
    ];

    Ok(Json(organizations))
}

#[derive(Debug, Deserialize)]
pub struct EditWorkflowTitleRequest {
    pub title: String,
}

pub async fn edit_workflow_title(
    Path(workflow_uuid): Path<String>,
    State(_state): State<AppState>,
    Json(payload): Json<EditWorkflowTitleRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validate title length
    if payload.title.trim().is_empty() {
        tracing::warn!(
            "Workflow {} title update failed: Title cannot be empty",
            workflow_uuid
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Title cannot be empty" })),
        ));
    }

    if payload.title.len() > 50 {
        tracing::warn!(
            "Workflow {} title update failed: Title length {} exceeds maximum of 50 characters",
            workflow_uuid,
            payload.title.len()
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Title cannot exceed 50 characters" })),
        ));
    }

    // Validate for invalid characters (control characters, invisible characters)
    // Check for control characters (except normal whitespace like space, tab, newline)
    // and invisible Unicode characters
    if payload.title.chars().any(|c| {
        // Control characters (except common whitespace)
        (c.is_control() && !matches!(c, '\t' | '\n' | '\r')) ||
        // Zero-width characters
        matches!(c, 
            '\u{200B}' | // Zero Width Space
            '\u{200C}' | // Zero Width Non-Joiner
            '\u{200D}' | // Zero Width Joiner
            '\u{FEFF}' | // Zero Width No-Break Space
            '\u{00AD}'   // Soft Hyphen
        ) ||
        // Bidirectional formatting characters
        matches!(c, '\u{200E}'..='\u{200F}' | '\u{202A}'..='\u{202E}') ||
        // Other invisible formatting characters
        matches!(c, '\u{2060}'..='\u{206F}')
    }) {
        tracing::warn!(
            "Workflow {} title update failed: Title contains invalid characters (control or invisible characters)",
            workflow_uuid
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Title contains invalid characters (control or invisible characters are not allowed)" })),
        ));
    }

    // Mock: Log the title change (in production, save to database)
    tracing::info!(
        "Workflow {} title updated successfully to: {}",
        workflow_uuid,
        payload.title
    );

    Ok(Json(json!({
        "message": "Title updated successfully",
        "workflow_uuid": workflow_uuid,
        "title": payload.title
    })))
}
