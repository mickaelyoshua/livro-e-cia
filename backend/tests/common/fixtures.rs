// Test fixtures: Factory functions for creating test data
//
// All fixtures create data directly in the database for integration testing.

use backend::{
    auth::password::hash_password,
    models::{Category, NewCategory, NewProduct, NewUser, Product, Role, User},
    schema::{categories, products, roles, users},
};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel::PgConnection;
use rust_decimal::Decimal;
use uuid::Uuid;

/// Default test password used for all test users
pub const TEST_PASSWORD: &str = "TestPassword123!";

/// Creates an admin user with verified email
pub fn create_admin_user(conn: &mut PgConnection) -> User {
    let admin_role_id = roles::table
        .filter(roles::name.eq("admin"))
        .select(roles::id)
        .first::<Uuid>(conn)
        .expect("Admin role must exist");

    let new_user = NewUser {
        email: format!("admin_{}@test.com", Uuid::new_v4()),
        password_hash: hash_password(TEST_PASSWORD).expect("Failed to hash password"),
        name: "Test Admin".to_string(),
        role_id: admin_role_id,
    };

    diesel::insert_into(users::table)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result(conn)
        .expect("Failed to create admin user")
}

/// Creates an admin user and sets email_verified to true
pub fn create_verified_admin(conn: &mut PgConnection) -> User {
    let user = create_admin_user(conn);

    diesel::update(users::table.find(user.id))
        .set((
            users::email_verified.eq(true),
            users::email_verified_at.eq(Some(Utc::now())),
        ))
        .returning(User::as_returning())
        .get_result(conn)
        .expect("Failed to verify admin user")
}

/// Creates an employee user with verified email
pub fn create_employee_user(conn: &mut PgConnection) -> User {
    let employee_role_id = roles::table
        .filter(roles::name.eq("employee"))
        .select(roles::id)
        .first::<Uuid>(conn)
        .expect("Employee role must exist");

    let new_user = NewUser {
        email: format!("employee_{}@test.com", Uuid::new_v4()),
        password_hash: hash_password(TEST_PASSWORD).expect("Failed to hash password"),
        name: "Test Employee".to_string(),
        role_id: employee_role_id,
    };

    let user = diesel::insert_into(users::table)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result(conn)
        .expect("Failed to create employee user");

    // Verify the email
    diesel::update(users::table.find(user.id))
        .set((
            users::email_verified.eq(true),
            users::email_verified_at.eq(Some(Utc::now())),
        ))
        .returning(User::as_returning())
        .get_result(conn)
        .expect("Failed to verify employee user")
}

/// Creates an inactive user
pub fn create_inactive_user(conn: &mut PgConnection) -> User {
    let employee_role_id = roles::table
        .filter(roles::name.eq("employee"))
        .select(roles::id)
        .first::<Uuid>(conn)
        .expect("Employee role must exist");

    let new_user = NewUser {
        email: format!("inactive_{}@test.com", Uuid::new_v4()),
        password_hash: hash_password(TEST_PASSWORD).expect("Failed to hash password"),
        name: "Inactive User".to_string(),
        role_id: employee_role_id,
    };

    let user = diesel::insert_into(users::table)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result(conn)
        .expect("Failed to create inactive user");

    // Set inactive and verify email
    diesel::update(users::table.find(user.id))
        .set((
            users::is_active.eq(false),
            users::email_verified.eq(true),
        ))
        .returning(User::as_returning())
        .get_result(conn)
        .expect("Failed to deactivate user")
}

/// Creates a user with unverified email
pub fn create_unverified_user(conn: &mut PgConnection) -> User {
    let employee_role_id = roles::table
        .filter(roles::name.eq("employee"))
        .select(roles::id)
        .first::<Uuid>(conn)
        .expect("Employee role must exist");

    let new_user = NewUser {
        email: format!("unverified_{}@test.com", Uuid::new_v4()),
        password_hash: hash_password(TEST_PASSWORD).expect("Failed to hash password"),
        name: "Unverified User".to_string(),
        role_id: employee_role_id,
    };

    diesel::insert_into(users::table)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result(conn)
        .expect("Failed to create unverified user")
}

/// Creates a user with a password reset token
pub fn create_user_with_reset_token(conn: &mut PgConnection, token_hash: &str) -> User {
    let employee_role_id = roles::table
        .filter(roles::name.eq("employee"))
        .select(roles::id)
        .first::<Uuid>(conn)
        .expect("Employee role must exist");

    let new_user = NewUser {
        email: format!("reset_{}@test.com", Uuid::new_v4()),
        password_hash: hash_password(TEST_PASSWORD).expect("Failed to hash password"),
        name: "Reset Token User".to_string(),
        role_id: employee_role_id,
    };

    let user = diesel::insert_into(users::table)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result(conn)
        .expect("Failed to create user");

    diesel::update(users::table.find(user.id))
        .set((
            users::email_verified.eq(true),
            users::password_reset_token.eq(Some(token_hash)),
            users::password_reset_expires_at.eq(Some(Utc::now() + Duration::hours(1))),
        ))
        .returning(User::as_returning())
        .get_result(conn)
        .expect("Failed to set reset token")
}

/// Creates a user with an expired password reset token
pub fn create_user_with_expired_reset_token(conn: &mut PgConnection, token_hash: &str) -> User {
    let employee_role_id = roles::table
        .filter(roles::name.eq("employee"))
        .select(roles::id)
        .first::<Uuid>(conn)
        .expect("Employee role must exist");

    let new_user = NewUser {
        email: format!("expired_reset_{}@test.com", Uuid::new_v4()),
        password_hash: hash_password(TEST_PASSWORD).expect("Failed to hash password"),
        name: "Expired Reset User".to_string(),
        role_id: employee_role_id,
    };

    let user = diesel::insert_into(users::table)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result(conn)
        .expect("Failed to create user");

    diesel::update(users::table.find(user.id))
        .set((
            users::email_verified.eq(true),
            users::password_reset_token.eq(Some(token_hash)),
            users::password_reset_expires_at.eq(Some(Utc::now() - Duration::hours(1))),
        ))
        .returning(User::as_returning())
        .get_result(conn)
        .expect("Failed to set expired reset token")
}

/// Creates a test category
pub fn create_category(conn: &mut PgConnection, name: &str) -> Category {
    let new_category = NewCategory {
        name: name.to_string(),
        description: Some(format!("{} books", name)),
    };

    diesel::insert_into(categories::table)
        .values(&new_category)
        .returning(Category::as_returning())
        .get_result(conn)
        .expect("Failed to create category")
}

/// Creates a default test category (Fiction)
pub fn create_default_category(conn: &mut PgConnection) -> Category {
    create_category(conn, &format!("Fiction_{}", Uuid::new_v4()))
}

/// Creates a test product in the given category
pub fn create_product(conn: &mut PgConnection, category_id: Uuid) -> Product {
    let new_product = NewProduct {
        title: format!("Test Book {}", Uuid::new_v4()),
        author: "Test Author".to_string(),
        price: Decimal::new(2999, 2), // 29.99
        stock_quantity: 10,
        publisher: Some("Test Publisher".to_string()),
        publication_date: None,
        category_id,
        description: Some("A test book for integration testing".to_string()),
        cover_image_url: None,
    };

    diesel::insert_into(products::table)
        .values(&new_product)
        .returning(Product::as_returning())
        .get_result(conn)
        .expect("Failed to create product")
}

/// Creates an inactive (soft-deleted) product
pub fn create_inactive_product(conn: &mut PgConnection, category_id: Uuid) -> Product {
    let product = create_product(conn, category_id);

    diesel::update(products::table.find(product.id))
        .set(products::is_active.eq(false))
        .returning(Product::as_returning())
        .get_result(conn)
        .expect("Failed to deactivate product")
}

/// Gets the admin role from the database
pub fn get_admin_role(conn: &mut PgConnection) -> Role {
    roles::table
        .filter(roles::name.eq("admin"))
        .first::<Role>(conn)
        .expect("Admin role must exist")
}

/// Gets the employee role from the database
pub fn get_employee_role(conn: &mut PgConnection) -> Role {
    roles::table
        .filter(roles::name.eq("employee"))
        .first::<Role>(conn)
        .expect("Employee role must exist")
}

/// Helper to hash a token for storage (simulates what the app does)
pub fn hash_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}
