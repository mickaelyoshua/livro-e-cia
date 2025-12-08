pub mod auth;
pub mod employees;
pub mod products;

// Re-export for easy mounting
pub use auth::*;
pub use employees::*;
pub use products::*;
