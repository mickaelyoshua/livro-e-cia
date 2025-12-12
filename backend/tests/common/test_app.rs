// TestApp: Integration test infrastructure for Rocket application
//
// Provides a configured Rocket client with database access for testing.

use backend::{
    auth::jwt::{generate_access_token, generate_refresh_token},
    config::Environment,
    db::{init_pool, DbPool},
    email::{EmailConfig, MockEmailService},
    schema::{categories, products, refresh_tokens, roles, users},
};
use diesel::prelude::*;
use diesel::r2d2::PooledConnection;
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use rocket::local::blocking::Client;
use std::env;
use uuid::Uuid;

const TEST_JWT_SECRET: &str = "test-jwt-secret-at-least-32-bytes-long!!";

/// Main test infrastructure struct
/// Provides access to Rocket client, database pool, and JWT secret
pub struct TestApp {
    pub client: Client,
    pub pool: DbPool,
    pub jwt_secret: String,
}

impl TestApp {
    /// Creates a new TestApp instance with a configured Rocket client
    ///
    /// Uses TEST_DATABASE_URL environment variable for database connection.
    /// Sets up mock email service and development environment.
    pub fn new() -> Self {
        // Load test environment
        dotenv::dotenv().ok();

        let database_url = env::var("TEST_DATABASE_URL")
            .or_else(|_| env::var("DATABASE_URL"))
            .expect("TEST_DATABASE_URL or DATABASE_URL must be set for integration tests");

        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| TEST_JWT_SECRET.to_string());

        // Create database pool
        let pool = init_pool(&database_url).expect("Failed to create test database pool");

        // Create mock email service
        let email_config = EmailConfig::from_env();
        let email_service: Box<dyn backend::email::EmailService> =
            Box::new(MockEmailService::new(email_config));

        // Build Rocket instance with test configuration
        let rocket = rocket::build()
            .manage(jwt_secret.clone())
            .manage(pool.clone())
            .manage(email_service)
            .manage(Environment::Development)
            .mount(
                "/",
                rocket::routes![
                    backend::routes::login,
                    backend::routes::get_current_user,
                    backend::routes::refresh_token,
                    backend::routes::logout,
                    backend::routes::list_products,
                    backend::routes::get_product,
                    backend::routes::create_product,
                    backend::routes::update_product,
                    backend::routes::delete_product,
                    backend::routes::list_employees,
                    backend::routes::get_employee,
                    backend::routes::create_employee,
                    backend::routes::update_employee,
                    backend::routes::delete_employee,
                    backend::routes::verify_email,
                    backend::routes::forgot_password,
                    backend::routes::reset_password,
                ],
            );

        // Create blocking client (tracked for cookies/sessions)
        let client = Client::tracked(rocket).expect("Failed to create Rocket test client");

        Self {
            client,
            pool,
            jwt_secret,
        }
    }

    /// Get a database connection from the pool
    pub fn db_conn(&self) -> PooledConnection<ConnectionManager<PgConnection>> {
        self.pool.get().expect("Failed to get database connection")
    }

    /// Reset database by truncating all tables in FK order
    ///
    /// Preserves roles (seeded data) but clears all other tables.
    pub fn reset_database(&self) {
        let mut conn = self.db_conn();

        // Truncate in FK dependency order (children first)
        diesel::delete(refresh_tokens::table)
            .execute(&mut conn)
            .expect("Failed to truncate refresh_tokens");

        diesel::delete(products::table)
            .execute(&mut conn)
            .expect("Failed to truncate products");

        diesel::delete(users::table)
            .execute(&mut conn)
            .expect("Failed to truncate users");

        diesel::delete(categories::table)
            .execute(&mut conn)
            .expect("Failed to truncate categories");

        // Note: We keep roles because they're seeded by migrations
        // and needed for creating users
    }

    /// Generate a valid access token for testing
    pub fn access_token_for(&self, user_id: Uuid, role: &str) -> String {
        generate_access_token(user_id, role, &self.jwt_secret)
            .expect("Failed to generate access token")
    }

    /// Generate a valid refresh token for testing
    pub fn refresh_token_for(&self, user_id: Uuid, role: &str) -> String {
        generate_refresh_token(user_id, role, &self.jwt_secret)
            .expect("Failed to generate refresh token")
    }

    /// Get the admin role ID from the database
    pub fn admin_role_id(&self) -> Uuid {
        let mut conn = self.db_conn();
        roles::table
            .filter(roles::name.eq("admin"))
            .select(roles::id)
            .first::<Uuid>(&mut conn)
            .expect("Admin role must exist (check migrations)")
    }

    /// Get the employee role ID from the database
    pub fn employee_role_id(&self) -> Uuid {
        let mut conn = self.db_conn();
        roles::table
            .filter(roles::name.eq("employee"))
            .select(roles::id)
            .first::<Uuid>(&mut conn)
            .expect("Employee role must exist (check migrations)")
    }
}

impl Default for TestApp {
    fn default() -> Self {
        Self::new()
    }
}
