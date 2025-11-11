use api::{create_app, AppState};

pub fn create_test_app() -> axum::Router {
    let jwt_secret = "test-secret-key".to_string();
    let app_state = AppState { jwt_secret };
    create_app(app_state)
}
