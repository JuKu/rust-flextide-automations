use axum::{
    extract::{Path, Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
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
    pub db_pool: flextide_core::database::DatabasePool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // email
    pub user_uuid: String, // user UUID
    pub exp: usize,
    pub iat: usize,
    pub is_server_admin: bool,
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

/// Helper function to create error response with CORS headers
fn error_response(status: StatusCode, error: Value) -> Response {
    let mut response = Json(error).into_response();
    *response.status_mut() = status;
    response.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        header::HeaderValue::from_static("*"),
    );
    response.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        header::HeaderValue::from_static("*"),
    );
    response.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        header::HeaderValue::from_static("*"),
    );
    response
}

/// Authentication middleware - validates JWT token
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();
    let method = request.method().clone();

    // Skip auth for OPTIONS requests (CORS preflight)
    if method == axum::http::Method::OPTIONS {
        tracing::debug!("[Auth] Skipping authentication for OPTIONS request to {}", path);
        return next.run(request).await;
    }

    // Skip auth for login and register endpoints
    if path == "/api/login" || path == "/api/register" || path == "/api/health" {
        tracing::debug!("[Auth] Skipping authentication for endpoint: {}", path);
        return next.run(request).await;
    }

    tracing::info!("[Auth] Authenticating request: {} {}", method, path);

    // Extract token from Authorization header
    let headers = request.headers();
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(header) => {
            match header.strip_prefix("Bearer ") {
                Some(t) => {
                    tracing::debug!("[Auth] Token found in Authorization header");
                    t
                }
                None => {
                    tracing::warn!("[Auth] Invalid Authorization header format for {} {}", method, path);
                    return error_response(
                        StatusCode::UNAUTHORIZED,
                        json!({ "error": "Invalid Authorization header format" }),
                    );
                }
            }
        }
        None => {
            tracing::warn!("[Auth] Missing Authorization header for {} {}", method, path);
            return error_response(
                StatusCode::UNAUTHORIZED,
                json!({ "error": "Missing Authorization header" }),
            );
        }
    };

    // Decode and validate token
    let token_data = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_ref()),
        &Validation::default(),
    ) {
        Ok(data) => {
            tracing::debug!("[Auth] Token decoded successfully for user: {}", data.claims.sub);
            data
        }
        Err(e) => {
            tracing::warn!("[Auth] Token decode failed for {} {}: {:?}", method, path, e);
            return error_response(
                StatusCode::UNAUTHORIZED,
                json!({ "error": "Invalid or expired token" }),
            );
        }
    };

    // Check if token is expired
    let now = Utc::now().timestamp() as usize;
    if token_data.claims.exp < now {
        tracing::warn!(
            "[Auth] Token expired for user {} (exp: {}, now: {})",
            token_data.claims.sub,
            token_data.claims.exp,
            now
        );
        return error_response(
            StatusCode::UNAUTHORIZED,
            json!({ "error": "Token expired" }),
        );
    }

    // Attach claims to request extensions for use in handlers
    request.extensions_mut().insert(token_data.claims.clone());
    tracing::info!(
        "[Auth] Authentication successful for user {} (is_server_admin: {})",
        token_data.claims.sub,
        token_data.claims.is_server_admin
    );

    next.run(request).await
}

/// Organization check middleware - validates user belongs to organization
pub async fn organization_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();
    let method = request.method().clone();

    // Skip for OPTIONS requests (CORS preflight)
    if method == axum::http::Method::OPTIONS {
        tracing::debug!("[Org] Skipping organization check for OPTIONS request to {}", path);
        return next.run(request).await;
    }

    // Skip for login, register, health, logout, and organizations/list-own endpoints
    if path == "/api/login"
        || path == "/api/register"
        || path == "/api/health"
        || path == "/api/logout"
        || path == "/api/organizations/list-own"
    {
        tracing::debug!("[Org] Skipping organization check for endpoint: {}", path);
        return next.run(request).await;
    }

    tracing::info!("[Org] Checking organization for request: {} {}", method, path);

    // Extract organization UUID from header
    let headers = request.headers();
    let org_uuid = match headers.get("X-Organization-UUID") {
        Some(header) => {
            match header.to_str() {
                Ok(uuid) => {
                    tracing::debug!("[Org] Organization UUID found in header: {}", uuid);
                    uuid
                }
                Err(e) => {
                    tracing::warn!("[Org] Invalid X-Organization-UUID header format for {} {}: {:?}", method, path, e);
                    return error_response(
                        StatusCode::BAD_REQUEST,
                        json!({ "error": "Invalid X-Organization-UUID header" }),
                    );
                }
            }
        }
        None => {
            tracing::warn!("[Org] Missing X-Organization-UUID header for {} {}", method, path);
            return error_response(
                StatusCode::BAD_REQUEST,
                json!({ 
                    "error": "Missing X-Organization-UUID header",
                    "code": "MISSING_ORG_UUID"
                }),
            );
        }
    };

    // Get user claims from request extensions (set by auth_middleware)
    let claims = match request.extensions().get::<Claims>() {
        Some(c) => {
            tracing::debug!("[Org] User claims found: user={}, is_server_admin={}", c.sub, c.is_server_admin);
            c
        }
        None => {
            tracing::error!("[Org] User not authenticated (claims missing) for {} {}", method, path);
            return error_response(
                StatusCode::UNAUTHORIZED,
                json!({ "error": "User not authenticated" }),
            );
        }
    };

    // Extract organization UUID before mutable borrow
    let org_uuid_string = org_uuid.to_string();

    // TODO: Check if user belongs to organization (database query)
    // For now, assume user belongs to organization
    // In production, query database to verify user membership
    let user_belongs_to_org = true; // Placeholder

    if !user_belongs_to_org {
        tracing::warn!(
            "[Org] User {} does not belong to organization {}",
            claims.sub,
            org_uuid_string
        );
        return error_response(
            StatusCode::FORBIDDEN,
            json!({ "error": "User does not belong to this organization" }),
        );
    }

    tracing::info!(
        "[Org] Organization check passed: user {} -> org {}",
        claims.sub,
        org_uuid_string
    );

    // Attach organization UUID to request extensions
    request.extensions_mut().insert(org_uuid_string);

    next.run(request).await
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
        .nest("/api", flextide_modules_crm::create_router())
        .layer(
            ServiceBuilder::new()
                .layer(cors)
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware,
                ))
                .layer(axum::middleware::from_fn(organization_middleware))
                .layer(trace_layer)
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
    // Get user from database by email
    let user = match flextide_core::user::get_user_by_email(&state.db_pool, &payload.email).await {
        Ok(user) => user,
        Err(flextide_core::user::UserDatabaseError::Sql(sqlx::Error::RowNotFound)) => {
            // User not found - return generic error to avoid email enumeration
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Invalid email or password" })),
            ));
        }
        Err(e) => {
            tracing::error!("Database error during login: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            ));
        }
    };

    // Verify password
    let password_valid = flextide_core::user::verify_password(&payload.password, &user.password_hash)
        .map_err(|e| {
            tracing::error!("Password verification error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            )
        })?;

    if !password_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid email or password" })),
        ));
    }

    // Check if account is activated
    if !user.activated {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Account is not activated" })),
        ));
    }

    // Generate JWT token
    let now = Utc::now();
    let exp = (now + Duration::hours(24)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    // Set server admin status (admin@example.com is server admin)
    let is_server_admin = payload.email == "admin@example.com";

    let claims = Claims {
        sub: payload.email.clone(),
        user_uuid: user.uuid.clone(),
        exp,
        iat,
        is_server_admin,
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

    // Set server admin status (admin@example.com is server admin)
    let is_server_admin = payload.email == "admin@example.com";

    let claims = Claims {
        sub: payload.email.clone(),
        user_uuid: user_uuid.clone(),
        exp,
        iat,
        is_server_admin,
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

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum License {
    Free,
    Pro,
    #[serde(rename = "Pro+")]
    ProPlus,
    Team,
}

#[derive(Debug, Serialize)]
pub struct Organization {
    pub uuid: String,
    pub title: String,
    pub is_admin: bool,
    #[serde(serialize_with = "serialize_license")]
    pub license: License,
}

fn serialize_license<S>(license: &License, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let s = match license {
        License::Free => "Free",
        License::Pro => "Pro",
        License::ProPlus => "Pro+",
        License::Team => "Team",
    };
    serializer.serialize_str(s)
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
            license: License::ProPlus,
        },
        Organization {
            uuid: uuid::Uuid::new_v4().to_string(),
            title: "Another Org".to_string(),
            is_admin: true,
            license: License::Team,
        },
        Organization {
            uuid: uuid::Uuid::new_v4().to_string(),
            title: "Test Org".to_string(),
            is_admin: true,
            license: License::Pro,
        },
        Organization {
            uuid: uuid::Uuid::new_v4().to_string(),
            title: "Test Org 2".to_string(),
            is_admin: false,
            license: License::Free,
        },
        Organization {
            uuid: uuid::Uuid::new_v4().to_string(),
            title: "Test Org 3".to_string(),
            is_admin: false,
            license: License::Pro,
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
