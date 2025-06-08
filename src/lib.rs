// Core modules
pub mod app;
pub mod auth;
pub mod config;
pub mod database;
pub mod handlers;
pub mod models;
pub mod schema;

// Re-export commonly used items for convenience
pub use app::create_app;
pub use config::AppConfig;
pub use database::connection::{AppState, create_diesel_pool};

// Re-export auth types for external usage
pub use auth::{AuthBackend, DieselSessionStore, SESSION_DURATION_DAYS};

// Common error type
pub type Result<T> = anyhow::Result<T>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_module_exports() {
        // Simple smoke test to ensure modules compile
        // More comprehensive tests will be in the tests/ directory
    }
}
