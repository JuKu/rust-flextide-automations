use axum::{
    extract::{Extension, Path, Query, Request, State},
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
use sqlx::Row;
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

// Re-export Claims from flextide-core for convenience
pub use flextide_core::jwt::Claims;

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
    State(state): State<AppState>,
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

    // Skip for login, register, health, logout, organizations/list-own, and organizations/create endpoints
    if path == "/api/login"
        || path == "/api/register"
        || path == "/api/health"
        || path == "/api/logout"
        || path == "/api/organizations/list-own"
        || path == "/api/organizations/create"
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
    
    // Attach database pool to request extensions for use in handlers
    request.extensions_mut().insert(state.db_pool.clone());

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
        .route("/api/organizations/create", post(create_organization))
        .route("/api/permissions", get(get_permissions))
        .route("/api/workflows/{workflow_uuid}/edit-title", post(edit_workflow_title))
        .route("/api/executions/last-executions", get(get_last_executions))
        .route("/api/integrations", get(get_integrations))
        .route("/api/integrations/list", get(list_integrations))
        .route("/api/integrations/search", get(search_integrations))
        .nest("/api", flextide_modules_crm::create_router())
        .layer(
            ServiceBuilder::new()
                .layer(cors)
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware,
                ))
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    organization_middleware,
                ))
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
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<Organization>>, (StatusCode, Json<Value>)> {
    use flextide_core::database::DatabasePool;
    
    let organizations = match &state.db_pool {
        DatabasePool::MySql(p) => {
            sqlx::query_as::<_, (String, String, String)>(
                "SELECT o.uuid, o.name, om.role
                 FROM organizations o
                 INNER JOIN organization_members om ON o.uuid = om.org_id
                 WHERE om.user_id = ?
                 ORDER BY o.name"
            )
            .bind(&claims.user_uuid)
            .fetch_all(p)
            .await
        }
        DatabasePool::Postgres(p) => {
            sqlx::query_as::<_, (String, String, String)>(
                "SELECT o.uuid, o.name, om.role
                 FROM organizations o
                 INNER JOIN organization_members om ON o.uuid = om.org_id
                 WHERE om.user_id = $1
                 ORDER BY o.name"
            )
            .bind(&claims.user_uuid)
            .fetch_all(p)
            .await
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query_as::<_, (String, String, String)>(
                "SELECT o.uuid, o.name, om.role
                 FROM organizations o
                 INNER JOIN organization_members om ON o.uuid = om.org_id
                 WHERE om.user_id = ?1
                 ORDER BY o.name"
            )
            .bind(&claims.user_uuid)
            .fetch_all(p)
            .await
        }
    }
    .map_err(|e| {
        tracing::error!("Failed to fetch organizations for user {}: {}", claims.user_uuid, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to fetch organizations" })),
        )
    })?;

    let result: Vec<Organization> = organizations
        .into_iter()
        .map(|(uuid, name, role)| {
            // User is admin if role is 'admin' or 'owner'
            let is_admin = role == "admin" || role == "owner";
            // TODO: Add license field to organizations table, defaulting to Free for now
        Organization {
                uuid,
                title: name,
                is_admin,
            license: License::Free,
            }
        })
        .collect();

    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct CreateOrganizationRequest {
    pub name: String,
}

/// Create a new organization and make the current user the owner
///
/// POST /api/organizations/create
/// Creates a new organization with the given name and adds the current user as owner.
/// Returns an error if the user already has 50 or more organizations.
pub async fn create_organization(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateOrganizationRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    use flextide_core::database::DatabasePool;
    use uuid::Uuid;

    // Validate organization name
    let name = payload.name.trim();
    if name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Organization name cannot be empty" })),
        ));
    }

    if name.len() > 255 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Organization name cannot exceed 255 characters" })),
        ));
    }

    // Check if user already has 50 or more organizations
    let count: i64 = match &state.db_pool {
        DatabasePool::MySql(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM organization_members WHERE user_id = ?")
                .bind(&claims.user_uuid)
                .fetch_one(p)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to count user organizations: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Database error" })),
                    )
                })?;
            row.get("count")
        }
        DatabasePool::Postgres(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM organization_members WHERE user_id = $1")
                .bind(&claims.user_uuid)
                .fetch_one(p)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to count user organizations: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Database error" })),
                    )
                })?;
            row.get("count")
        }
        DatabasePool::Sqlite(p) => {
            let row = sqlx::query("SELECT COUNT(*) as count FROM organization_members WHERE user_id = ?1")
                .bind(&claims.user_uuid)
                .fetch_one(p)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to count user organizations: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Database error" })),
                    )
                })?;
            row.get("count")
        }
    };

    if count >= 50 {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "You cannot have more than 50 organizations" })),
        ));
    }

    // Generate organization UUID
    let org_uuid = Uuid::new_v4().to_string();

    // Create organization in a transaction
    match &state.db_pool {
        DatabasePool::MySql(p) => {
            let mut tx = p.begin().await.map_err(|e| {
                tracing::error!("Failed to start transaction: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Database error" })),
                )
            })?;

            // Insert organization
            sqlx::query("INSERT INTO organizations (uuid, name, owner_user_id) VALUES (?, ?, ?)")
                .bind(&org_uuid)
                .bind(&name)
                .bind(&claims.user_uuid)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to create organization: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Failed to create organization" })),
                    )
                })?;

            // Add user as owner
            sqlx::query("INSERT INTO organization_members (org_id, user_id, role) VALUES (?, ?, 'owner')")
                .bind(&org_uuid)
                .bind(&claims.user_uuid)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to add user as owner: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Failed to add user as owner" })),
                    )
                })?;

            // Grant super_admin permission to the user for the organization
            sqlx::query(
                "INSERT INTO user_permissions (user_id, organization_uuid, permission_name)
                 VALUES (?, ?, 'super_admin')
                 ON DUPLICATE KEY UPDATE permission_name = permission_name",
            )
            .bind(&claims.user_uuid)
            .bind(&org_uuid)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("Failed to grant super_admin permission: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to grant super_admin permission" })),
                )
            })?;

            tx.commit().await.map_err(|e| {
                tracing::error!("Failed to commit transaction: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Database error" })),
                )
            })?;
        }
        DatabasePool::Postgres(p) => {
            let mut tx = p.begin().await.map_err(|e| {
                tracing::error!("Failed to start transaction: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Database error" })),
                )
            })?;

            // Insert organization
            sqlx::query("INSERT INTO organizations (uuid, name, owner_user_id) VALUES ($1, $2, $3)")
                .bind(&org_uuid)
                .bind(&name)
                .bind(&claims.user_uuid)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to create organization: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Failed to create organization" })),
                    )
                })?;

            // Add user as owner
            sqlx::query("INSERT INTO organization_members (org_id, user_id, role) VALUES ($1, $2, 'owner')")
                .bind(&org_uuid)
                .bind(&claims.user_uuid)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to add user as owner: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Failed to add user as owner" })),
                    )
                })?;

            // Grant super_admin permission to the user for the organization
            sqlx::query(
                "INSERT INTO user_permissions (user_id, organization_uuid, permission_name)
                 VALUES ($1, $2, 'super_admin')
                 ON CONFLICT (user_id, organization_uuid, permission_name) DO NOTHING",
            )
            .bind(&claims.user_uuid)
            .bind(&org_uuid)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("Failed to grant super_admin permission: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to grant super_admin permission" })),
                )
            })?;

            tx.commit().await.map_err(|e| {
                tracing::error!("Failed to commit transaction: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Database error" })),
                )
            })?;
        }
        DatabasePool::Sqlite(p) => {
            let mut tx = p.begin().await.map_err(|e| {
                tracing::error!("Failed to start transaction: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Database error" })),
                )
            })?;

            // Insert organization
            sqlx::query("INSERT INTO organizations (uuid, name, owner_user_id) VALUES (?1, ?2, ?3)")
                .bind(&org_uuid)
                .bind(&name)
                .bind(&claims.user_uuid)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to create organization: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Failed to create organization" })),
                    )
                })?;

            // Add user as owner
            sqlx::query("INSERT INTO organization_members (org_id, user_id, role) VALUES (?1, ?2, 'owner')")
                .bind(&org_uuid)
                .bind(&claims.user_uuid)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to add user as owner: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Failed to add user as owner" })),
                    )
                })?;

            // Grant super_admin permission to the user for the organization
            sqlx::query(
                "INSERT OR IGNORE INTO user_permissions (user_id, organization_uuid, permission_name)
                 VALUES (?1, ?2, 'super_admin')",
            )
            .bind(&claims.user_uuid)
            .bind(&org_uuid)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("Failed to grant super_admin permission: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to grant super_admin permission" })),
                )
            })?;

            tx.commit().await.map_err(|e| {
                tracing::error!("Failed to commit transaction: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Database error" })),
                )
            })?;
        }
    }

    tracing::info!(
        "Organization '{}' created successfully with UUID {} for user {}. User granted super_admin permission.",
        name,
        org_uuid,
        claims.user_uuid
    );

    Ok(Json(json!({
        "uuid": org_uuid,
        "name": name,
        "is_admin": true,
        "license": "Free"
    })))
}

/// Get all permissions for the current user in the current organization
///
/// GET /api/permissions
/// Returns a list of permission strings that the user has for the current organization
pub async fn get_permissions(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Extension(org_uuid): Extension<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    use flextide_core::permissions::list_user_permissions;
    
    let user_permissions = list_user_permissions(&state.db_pool, &claims.user_uuid, &org_uuid)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch user permissions: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to fetch permissions" })),
            )
        })?;
    
    // Extract just the permission names
    let permissions: Vec<String> = user_permissions
        .into_iter()
        .map(|up| up.permission_name)
        .collect();
    
    Ok(Json(json!({
        "permissions": permissions,
        "user_uuid": claims.user_uuid,
        "organization_uuid": org_uuid
    })))
}

#[derive(Debug, Deserialize)]
pub struct LastExecutionsQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_page() -> u32 {
    1
}

fn default_limit() -> u32 {
    30
}

/// Helper function to extract execution data from a database row
/// Works with all database types (MySQL, PostgreSQL, SQLite)
fn extract_execution_from_row<R: Row>(row: R) -> ExecutionResponse
where
    usize: sqlx::ColumnIndex<R>,
    for<'r> &'r str: sqlx::ColumnIndex<R>,
    for<'r> String: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    for<'r> Option<String>: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    for<'r> Value: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    for<'r> Option<Value>: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
{
    let uuid: String = row.get(0usize);
    let status: String = row.get(1usize);
    let workflow_name: String = row.get(2usize);
    let workflow_uuid: String = row.get(3usize);
    let started_at: String = row.get(4usize);
    // Handle NULL dates - DATE_FORMAT/TO_CHAR return NULL for NULL input
    // Use try_get to safely handle NULL values
    let finished_at: Option<String> = row.try_get::<Option<String>, _>(5usize)
        .ok()
        .flatten()
        .filter(|s| !s.is_empty());
    let trigger_type: String = row.get(6usize);
    
    // Handle JSON metadata - try to get as Value first, then as String
    let metadata_value: Option<Value> = row
        .try_get::<Option<Value>, _>(7usize)
        .ok()
        .flatten()
        .or_else(|| {
            row.try_get::<Option<String>, _>(7usize)
                .ok()
                .flatten()
                .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        });

    // Create short UUID (first 8 characters)
    let short_uuid = uuid.chars().take(8).collect::<String>();

    ExecutionResponse {
        uuid,
        short_uuid,
        status,
        workflow_name,
        workflow_uuid,
        started_at,
        finished_at,
        trigger_type,
        credits_used: 0, // TODO: Add credits tracking
        metadata: metadata_value,
    }
}

#[derive(Debug, Serialize)]
pub struct ExecutionResponse {
    pub uuid: String,
    pub short_uuid: String,
    pub status: String,
    pub workflow_name: String,
    pub workflow_uuid: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub trigger_type: String,
    pub credits_used: i64, // TODO: Add credits tracking later, defaulting to 0
    pub metadata: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct LastExecutionsResponse {
    pub executions: Vec<ExecutionResponse>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

/// Get last executions for the organization
///
/// GET /api/executions/last-executions?page=1&limit=30
pub async fn get_last_executions(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Extension(org_uuid): Extension<String>,
    Query(query): Query<LastExecutionsQuery>,
) -> Result<Json<LastExecutionsResponse>, (StatusCode, Json<Value>)> {
    use flextide_core::database::DatabasePool;
    use flextide_core::user::{user_belongs_to_organization, user_has_permission};

    // Check if user belongs to organization
    let belongs = user_belongs_to_organization(&state.db_pool, &claims.user_uuid, &org_uuid)
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
    let has_permission = user_has_permission(
        &state.db_pool,
        &claims.user_uuid,
        &org_uuid,
        "can_see_last_executions",
    )
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
            Json(json!({
                "error": "User does not have permission to see last executions"
            })),
        ));
    }

    // Validate limit (max 50)
    let limit = query.limit.min(50).max(1);
    let page = query.page.max(1);
    let offset = (page - 1) * limit;

    // Get total count
    let total = match &state.db_pool {
        DatabasePool::MySql(p) => {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM runs WHERE organization_uuid = ?"
            )
            .bind(&org_uuid)
            .fetch_one(p)
            .await
        }
        DatabasePool::Postgres(p) => {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM runs WHERE organization_uuid = $1"
            )
            .bind(&org_uuid)
            .fetch_one(p)
            .await
        }
        DatabasePool::Sqlite(p) => {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM runs WHERE organization_uuid = ?1"
            )
            .bind(&org_uuid)
            .fetch_one(p)
            .await
        }
    }
    .map_err(|e| {
        tracing::error!("Failed to count executions: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to fetch executions" })),
        )
    })?;

    // Fetch executions with workflow name
    // Using a helper function to handle different database types
    let execution_responses: Vec<ExecutionResponse> = match &state.db_pool {
        DatabasePool::MySql(p) => {
            let rows = sqlx::query(
                "SELECT 
                    r.uuid,
                    r.status,
                    COALESCE(SUBSTRING(w.name, 1, 50), 'Unknown') as workflow_name,
                    r.workflow_id,
                    DATE_FORMAT(r.started_at, '%Y-%m-%d %H:%i:%s') as started_at,
                    DATE_FORMAT(r.finished_at, '%Y-%m-%d %H:%i:%s') as finished_at,
                    r.trigger_type,
                    r.metadata
                 FROM runs r
                 LEFT JOIN workflows w ON r.workflow_id = w.uuid
                 WHERE r.organization_uuid = ?
                 ORDER BY r.created_at DESC
                 LIMIT ? OFFSET ?"
            )
            .bind(&org_uuid)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(p)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch executions: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to fetch executions" })),
                )
            })?;

            rows.into_iter()
                .map(|row| extract_execution_from_row(row))
                .collect()
        }
        DatabasePool::Postgres(p) => {
            let rows = sqlx::query(
                "SELECT 
                    r.uuid,
                    r.status,
                    COALESCE(SUBSTRING(w.name, 1, 50), 'Unknown') as workflow_name,
                    r.workflow_id,
                    TO_CHAR(r.started_at, 'YYYY-MM-DD HH24:MI:SS') as started_at,
                    TO_CHAR(r.finished_at, 'YYYY-MM-DD HH24:MI:SS') as finished_at,
                    r.trigger_type,
                    r.metadata
                 FROM runs r
                 LEFT JOIN workflows w ON r.workflow_id = w.uuid
                 WHERE r.organization_uuid = $1
                 ORDER BY r.created_at DESC
                 LIMIT $2 OFFSET $3"
            )
            .bind(&org_uuid)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(p)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch executions: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to fetch executions" })),
                )
            })?;

            rows.into_iter()
                .map(|row| extract_execution_from_row(row))
                .collect()
        }
        DatabasePool::Sqlite(p) => {
            let rows = sqlx::query(
                "SELECT 
                    r.uuid,
                    r.status,
                    COALESCE(SUBSTRING(w.name, 1, 50), 'Unknown') as workflow_name,
                    r.workflow_id,
                    strftime('%Y-%m-%d %H:%M:%S', r.started_at) as started_at,
                    strftime('%Y-%m-%d %H:%M:%S', r.finished_at) as finished_at,
                    r.trigger_type,
                    r.metadata
                 FROM runs r
                 LEFT JOIN workflows w ON r.workflow_id = w.uuid
                 WHERE r.organization_uuid = ?1
                 ORDER BY r.created_at DESC
                 LIMIT ?2 OFFSET ?3"
            )
            .bind(&org_uuid)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(p)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch executions: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to fetch executions" })),
                )
            })?;

            rows.into_iter()
                .map(|row| extract_execution_from_row(row))
                .collect()
        }
    };

    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    Ok(Json(LastExecutionsResponse {
        executions: execution_responses,
        total,
        page,
        limit,
        total_pages,
    }))
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

/// Get activated integrations
///
/// GET /api/integrations
/// Returns a list of activated integrations with their frontend routes
pub async fn get_integrations(
    State(_state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Extension(_org_uuid): Extension<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Default activated integrations
    let integrations = json!([
        {
            "name": "JIRA",
            "route": "/integrations/jira/overview"
        },
        {
            "name": "GitHub Issues",
            "route": "/integrations/github-issues/overview"
        },
        {
            "name": "OpenAI",
            "route": "/integrations/openai/overview"
        }
    ]);

    Ok(Json(integrations))
}

#[derive(Debug, Deserialize)]
pub struct ListIntegrationsQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

/// List all available integrations with pagination
///
/// GET /api/integrations/list?page=1&limit=20
/// Returns a paginated list of all integrations (activated and not activated)
pub async fn list_integrations(
    Query(query): Query<ListIntegrationsQuery>,
    State(_state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Extension(_org_uuid): Extension<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let page = query.page.max(1);
    let limit = query.limit.min(100).max(1);
    let offset = (page - 1) * limit;

    // Mock data - in production, this would come from database
    // This includes all available integrations
    let all_integrations = vec![
        json!({
            "uuid": "550e8400-e29b-41d4-a716-446655440001",
            "title": "JIRA",
            "description": "Integrate with JIRA to create, update, and manage issues. Track project progress and automate workflows.",
            "activated": true,
            "purchased": true,
            "author_name": "Flextide Team",
            "author_url": "https://flextide.com",
            "created_at": "2024-01-15T10:00:00Z",
            "updated_at": "2024-12-01T14:30:00Z",
            "version": "1.0.0",
            "verified": true,
            "third_party": false,
            "image_url": null,
            "image_description": null,
            "rating": 4.5,
            "configuration_url": "/integrations/jira/overview",
            "pricing_type": "free"
        }),
        json!({
            "uuid": "550e8400-e29b-41d4-a716-446655440002",
            "title": "GitHub Issues",
            "description": "Connect to GitHub to manage issues, pull requests, and repositories. Automate your development workflow.",
            "activated": true,
            "purchased": true,
            "author_name": "Flextide Team",
            "author_url": "https://flextide.com",
            "created_at": "2024-02-01T09:00:00Z",
            "updated_at": "2024-11-15T16:20:00Z",
            "version": "1.2.0",
            "verified": true,
            "third_party": false,
            "image_url": null,
            "image_description": null,
            "rating": 4.8,
            "configuration_url": "/integrations/github-issues/overview",
            "pricing_type": "free"
        }),
        json!({
            "uuid": "550e8400-e29b-41d4-a716-446655440003",
            "title": "OpenAI",
            "description": "Integrate OpenAI's GPT models for AI-powered automation. Generate content, analyze data, and create intelligent workflows.",
            "activated": true,
            "purchased": true,
            "author_name": "Flextide Team",
            "author_url": "https://flextide.com",
            "created_at": "2024-03-10T11:00:00Z",
            "updated_at": "2024-12-10T10:15:00Z",
            "version": "2.1.0",
            "verified": true,
            "third_party": false,
            "image_url": null,
            "image_description": null,
            "rating": 5.0,
            "configuration_url": "/integrations/openai/overview",
            "pricing_type": "free"
        }),
        json!({
            "uuid": "550e8400-e29b-41d4-a716-446655440008",
            "title": "Google Sheets",
            "description": "Read and write data to Google Sheets. Automate spreadsheet operations and data synchronization.",
            "activated": false,
            "purchased": true,
            "author_name": "Flextide Team",
            "author_url": "https://flextide.com",
            "created_at": "2024-08-15T09:00:00Z",
            "updated_at": "2024-12-08T10:00:00Z",
            "version": "1.1.0",
            "verified": true,
            "third_party": false,
            "image_url": null,
            "image_description": null,
            "rating": 4.4,
            "configuration_url": "/integrations/google-sheets/overview",
            "pricing_type": "free"
        }),
    ];

    let total = all_integrations.len() as u32;
    let start = offset as usize;
    let end = (offset + limit) as usize;
    let paginated_integrations: Vec<Value> = all_integrations
        .into_iter()
        .skip(start)
        .take((end - start).min(limit as usize))
        .collect();

    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    Ok(Json(json!({
        "integrations": paginated_integrations,
        "total": total,
        "page": page,
        "limit": limit,
        "total_pages": total_pages
    })))
}

#[derive(Debug, Deserialize)]
pub struct SearchIntegrationsQuery {
    pub q: String,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

/// Search integrations
///
/// GET /api/integrations/search?q=query&page=1&limit=20
/// Returns a paginated list of integrations matching the search query
pub async fn search_integrations(
    Query(query): Query<SearchIntegrationsQuery>,
    State(_state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Extension(_org_uuid): Extension<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let search_query = query.q.trim().to_lowercase();
    
    if search_query.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Search query cannot be empty" })),
        ));
    }

    let page = query.page.max(1);
    let limit = query.limit.min(100).max(1);
    let offset = (page - 1) * limit;

    // Mock data - same as list_integrations
    let all_integrations = vec![
        json!({
            "uuid": "550e8400-e29b-41d4-a716-446655440001",
            "title": "JIRA",
            "description": "Integrate with JIRA to create, update, and manage issues. Track project progress and automate workflows.",
            "activated": true,
            "purchased": true,
            "author_name": "Flextide Team",
            "author_url": "https://flextide.com",
            "created_at": "2024-01-15T10:00:00Z",
            "updated_at": "2024-12-01T14:30:00Z",
            "version": "1.0.0",
            "verified": true,
            "third_party": false,
            "image_url": null,
            "image_description": null,
            "rating": 4.5,
            "configuration_url": "/integrations/jira/overview",
            "pricing_type": "free"
        }),
        json!({
            "uuid": "550e8400-e29b-41d4-a716-446655440002",
            "title": "GitHub Issues",
            "description": "Connect to GitHub to manage issues, pull requests, and repositories. Automate your development workflow.",
            "activated": true,
            "purchased": true,
            "author_name": "Flextide Team",
            "author_url": "https://flextide.com",
            "created_at": "2024-02-01T09:00:00Z",
            "updated_at": "2024-11-15T16:20:00Z",
            "version": "1.2.0",
            "verified": true,
            "third_party": false,
            "image_url": null,
            "image_description": null,
            "rating": 4.8,
            "configuration_url": "/integrations/github-issues/overview",
            "pricing_type": "free"
        }),
        json!({
            "uuid": "550e8400-e29b-41d4-a716-446655440003",
            "title": "OpenAI",
            "description": "Integrate OpenAI's GPT models for AI-powered automation. Generate content, analyze data, and create intelligent workflows.",
            "activated": true,
            "purchased": true,
            "author_name": "Flextide Team",
            "author_url": "https://flextide.com",
            "created_at": "2024-03-10T11:00:00Z",
            "updated_at": "2024-12-10T10:15:00Z",
            "version": "2.1.0",
            "verified": true,
            "third_party": false,
            "image_url": null,
            "image_description": null,
            "rating": 5.0,
            "configuration_url": "/integrations/openai/overview",
            "pricing_type": "free"
        }),
        json!({
            "uuid": "550e8400-e29b-41d4-a716-446655440008",
            "title": "Google Sheets",
            "description": "Read and write data to Google Sheets. Automate spreadsheet operations and data synchronization.",
            "activated": false,
            "purchased": true,
            "author_name": "Flextide Team",
            "author_url": "https://flextide.com",
            "created_at": "2024-08-15T09:00:00Z",
            "updated_at": "2024-12-08T10:00:00Z",
            "version": "1.1.0",
            "verified": true,
            "third_party": false,
            "image_url": null,
            "image_description": null,
            "rating": 4.4,
            "configuration_url": "/integrations/google-sheets/overview",
            "pricing_type": "free"
        }),
    ];

    // Filter integrations by search query (search in title and description)
    let filtered: Vec<Value> = all_integrations
        .into_iter()
        .filter(|integration| {
            let title = integration.get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_lowercase();
            let description = integration.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_lowercase();
            title.contains(&search_query) || description.contains(&search_query)
        })
        .collect();

    let total = filtered.len() as u32;
    let start = offset as usize;
    let end = (offset + limit) as usize;
    let paginated_integrations: Vec<Value> = filtered
        .into_iter()
        .skip(start)
        .take((end - start).min(limit as usize))
        .collect();

    let total_pages = if total > 0 {
        ((total as f64) / (limit as f64)).ceil() as u32
    } else {
        0
    };

    Ok(Json(json!({
        "integrations": paginated_integrations,
        "total": total,
        "page": page,
        "limit": limit,
        "total_pages": total_pages
    })))
}
