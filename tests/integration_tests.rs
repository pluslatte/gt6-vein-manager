// Integration tests for the application
mod common;

use gt6_vein_manager::{create_app, AppConfig};
use common::{setup_test_app_state, setup_test_config};

#[tokio::test]
async fn test_app_creation() {
    // Test that the app can be created without panicking
    let state = setup_test_app_state().await.expect("Failed to setup test state");
    let app = create_app(state).await.expect("Failed to create app");
    
    // Basic smoke test - app should be created successfully
    assert!(!format!("{:?}", app).is_empty());
}

#[tokio::test]
async fn test_config_loading() {
    // Test configuration loading
    let config = setup_test_config();
    assert!(!config.port.is_empty());
    assert!(!config.database_url.is_empty());
    
    let addr = config.server_address();
    assert!(addr.starts_with("0.0.0.0:"));
}

// Add more integration tests here as needed
// Examples:
// - test_auth_endpoints
// - test_vein_crud_operations
// - test_middleware_behavior
