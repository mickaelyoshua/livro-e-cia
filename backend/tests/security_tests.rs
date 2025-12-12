// Security-focused integration tests
//
// Tests for: authorization bypass, token confusion, enumeration prevention,
// information disclosure, and other security concerns

mod common;

use common::{
    basic_auth_header, bearer_header, create_default_category, create_employee_user,
    create_inactive_user, create_product, create_verified_admin, expired_access_token,
    invalid_auth_header, token_with_wrong_secret, TestApp, TEST_PASSWORD,
};
use rocket::http::{ContentType, Status};
use serial_test::serial;
use shared::{LoginRequest, PaginatedResponse};

// ============================================
// Authorization Bypass Prevention
// ============================================

#[test]
#[serial]
fn admin_endpoints_reject_employee_token() {
    let app = TestApp::new();
    app.reset_database();

    let employee = create_employee_user(&mut app.db_conn());
    let category = create_default_category(&mut app.db_conn());
    let product = create_product(&mut app.db_conn(), category.id);
    let employee_role_id = app.employee_role_id();

    let token = app.access_token_for(employee.id, "employee");

    // Test all admin-only endpoints
    let admin_endpoints = [
        ("POST", "/api/v1/products"),
        ("PUT", &format!("/api/v1/products/{}", product.id)),
        ("DELETE", &format!("/api/v1/products/{}", product.id)),
        ("GET", "/api/v1/employees"),
        ("GET", &format!("/api/v1/employees/{}", employee.id)),
        ("POST", "/api/v1/employees"),
        ("PUT", &format!("/api/v1/employees/{}", employee.id)),
        ("DELETE", &format!("/api/v1/employees/{}", employee.id)),
    ];

    for (method, path) in admin_endpoints {
        let response = match method {
            "GET" => app
                .client
                .get(path.to_string())
                .header(bearer_header(&token))
                .dispatch(),
            "POST" => {
                let body = if path.contains("products") {
                    serde_json::json!({
                        "title": "Test",
                        "author": "Test",
                        "price": "9.99",
                        "stock_quantity": 1,
                        "category_id": category.id
                    })
                } else {
                    serde_json::json!({
                        "email": "test@example.com",
                        "name": "Test",
                        "role_id": employee_role_id
                    })
                };
                app.client
                    .post(path.to_string())
                    .header(bearer_header(&token))
                    .header(ContentType::JSON)
                    .body(serde_json::to_string(&body).unwrap())
                    .dispatch()
            }
            "PUT" => {
                let body = if path.contains("products") {
                    serde_json::json!({ "title": "Updated" })
                } else {
                    serde_json::json!({ "name": "Updated" })
                };
                app.client
                    .put(path.to_string())
                    .header(bearer_header(&token))
                    .header(ContentType::JSON)
                    .body(serde_json::to_string(&body).unwrap())
                    .dispatch()
            }
            "DELETE" => app
                .client
                .delete(path.to_string())
                .header(bearer_header(&token))
                .dispatch(),
            _ => unreachable!(),
        };

        assert_eq!(
            response.status(),
            Status::Forbidden,
            "Employee should get 403 for {} {}",
            method,
            path
        );
    }
}

// ============================================
// Token Confusion Prevention
// ============================================

#[test]
#[serial]
fn refresh_token_cannot_be_used_as_access_token() {
    let app = TestApp::new();
    app.reset_database();

    let user = create_verified_admin(&mut app.db_conn());
    let refresh_token = app.refresh_token_for(user.id, "admin");

    // Try to access /me with refresh token
    let response = app
        .client
        .get("/api/v1/auth/me")
        .header(bearer_header(&refresh_token))
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
#[serial]
fn access_token_cannot_be_used_as_refresh_token() {
    let app = TestApp::new();
    app.reset_database();

    let user = create_verified_admin(&mut app.db_conn());
    let access_token = app.access_token_for(user.id, "admin");

    let refresh_request = serde_json::json!({ "refresh_token": access_token });

    let response = app
        .client
        .post("/api/v1/auth/refresh")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&refresh_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

// ============================================
// Information Disclosure Prevention
// ============================================

#[test]
#[serial]
fn password_hash_never_in_user_responses() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let _employee = create_employee_user(&mut app.db_conn());

    let token = app.access_token_for(admin.id, "admin");

    // Check /me endpoint
    let me_response = app
        .client
        .get("/api/v1/auth/me")
        .header(bearer_header(&token))
        .dispatch();

    let me_body: serde_json::Value = me_response.into_json().unwrap();
    assert!(me_body.get("password_hash").is_none());
    assert!(me_body.get("passwordHash").is_none());

    // Check /employees endpoint
    let list_response = app
        .client
        .get("/api/v1/employees")
        .header(bearer_header(&token))
        .dispatch();

    let list_body: serde_json::Value = list_response.into_json().unwrap();
    for emp in list_body["data"].as_array().unwrap() {
        assert!(emp.get("password_hash").is_none());
        assert!(emp.get("passwordHash").is_none());
    }

    // Check login response
    let login_request = LoginRequest {
        email: admin.email,
        password: TEST_PASSWORD.to_string(),
    };

    let login_response = app
        .client
        .post("/api/v1/auth/login")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&login_request).unwrap())
        .dispatch();

    let login_body: serde_json::Value = login_response.into_json().unwrap();
    assert!(login_body["user"].get("password_hash").is_none());
    assert!(login_body["user"].get("passwordHash").is_none());
}

#[test]
#[serial]
fn login_same_error_for_invalid_email_and_password() {
    let app = TestApp::new();
    app.reset_database();

    let user = create_verified_admin(&mut app.db_conn());

    // Invalid email
    let invalid_email_request = LoginRequest {
        email: "nonexistent@example.com".to_string(),
        password: TEST_PASSWORD.to_string(),
    };

    let response1 = app
        .client
        .post("/api/v1/auth/login")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&invalid_email_request).unwrap())
        .dispatch();

    let body1: serde_json::Value = response1.into_json().unwrap();

    // Invalid password
    let invalid_password_request = LoginRequest {
        email: user.email,
        password: "WrongPassword123!".to_string(),
    };

    let response2 = app
        .client
        .post("/api/v1/auth/login")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&invalid_password_request).unwrap())
        .dispatch();

    let body2: serde_json::Value = response2.into_json().unwrap();

    // Both should have the same generic error message
    assert_eq!(body1["error"], body2["error"]);
    assert_eq!(body1["error"], "Invalid credentials");
}

#[test]
#[serial]
fn forgot_password_same_response_for_existing_and_nonexistent() {
    let app = TestApp::new();
    app.reset_database();

    let user = create_verified_admin(&mut app.db_conn());

    // Existing email
    let existing_request = serde_json::json!({ "email": user.email });
    let response1 = app
        .client
        .post("/api/v1/auth/forgot-password")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&existing_request).unwrap())
        .dispatch();

    let body1: serde_json::Value = response1.into_json().unwrap();

    // Non-existing email
    let nonexistent_request = serde_json::json!({ "email": "nonexistent@example.com" });
    let response2 = app
        .client
        .post("/api/v1/auth/forgot-password")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&nonexistent_request).unwrap())
        .dispatch();

    let body2: serde_json::Value = response2.into_json().unwrap();

    // Both should have the same response message
    assert_eq!(body1["message"], body2["message"]);
}

// ============================================
// Token Integrity Validation
// ============================================

#[test]
#[serial]
fn jwt_with_wrong_secret_rejected() {
    let app = TestApp::new();
    app.reset_database();

    let user = create_verified_admin(&mut app.db_conn());
    let bad_token = token_with_wrong_secret(user.id, "admin");

    let response = app
        .client
        .get("/api/v1/auth/me")
        .header(bearer_header(&bad_token))
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
#[serial]
fn expired_token_rejected() {
    let app = TestApp::new();
    app.reset_database();

    let user = create_verified_admin(&mut app.db_conn());
    let expired = expired_access_token(user.id, "admin", &app.jwt_secret);

    let response = app
        .client
        .get("/api/v1/auth/me")
        .header(bearer_header(&expired))
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
#[serial]
fn malformed_authorization_header_rejected() {
    let app = TestApp::new();
    app.reset_database();

    // Missing "Bearer " prefix
    let response = app
        .client
        .get("/api/v1/auth/me")
        .header(invalid_auth_header("some-token"))
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);

    // Wrong scheme (Basic instead of Bearer)
    let response2 = app
        .client
        .get("/api/v1/auth/me")
        .header(basic_auth_header("some-token"))
        .dispatch();

    assert_eq!(response2.status(), Status::Unauthorized);
}

// ============================================
// Account Lifecycle Security
// ============================================

#[test]
#[serial]
fn deleted_user_tokens_become_invalid() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let employee = create_employee_user(&mut app.db_conn());

    let admin_token = app.access_token_for(admin.id, "admin");
    let employee_token = app.access_token_for(employee.id, "employee");

    // Verify employee can access their data
    let before_delete = app
        .client
        .get("/api/v1/auth/me")
        .header(bearer_header(&employee_token))
        .dispatch();
    assert_eq!(before_delete.status(), Status::Ok);

    // Admin deletes the employee
    let delete_response = app
        .client
        .delete(format!("/api/v1/employees/{}", employee.id))
        .header(bearer_header(&admin_token))
        .dispatch();
    assert_eq!(delete_response.status(), Status::Ok);

    // Employee's token should now fail (user is inactive)
    let after_delete = app
        .client
        .get("/api/v1/auth/me")
        .header(bearer_header(&employee_token))
        .dispatch();

    // Could be 401 or 404 depending on implementation
    assert!(
        after_delete.status() == Status::Unauthorized
            || after_delete.status() == Status::NotFound
    );
}

#[test]
#[serial]
fn inactive_user_cannot_login() {
    let app = TestApp::new();
    app.reset_database();

    let inactive = create_inactive_user(&mut app.db_conn());

    let login_request = LoginRequest {
        email: inactive.email,
        password: TEST_PASSWORD.to_string(),
    };

    let response = app
        .client
        .post("/api/v1/auth/login")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&login_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Forbidden);
}

// ============================================
// SQL Injection Prevention
// ============================================

#[test]
#[serial]
fn sql_injection_in_search_params_prevented() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let category = create_default_category(&mut app.db_conn());
    let _product = create_product(&mut app.db_conn(), category.id);

    let token = app.access_token_for(admin.id, "admin");

    // Attempt SQL injection in search parameter
    let injection_attempts = [
        "'; DROP TABLE products; --",
        "1' OR '1'='1",
        "1; SELECT * FROM users; --",
        "' UNION SELECT * FROM users --",
    ];

    for attempt in injection_attempts {
        // URL encode manually - replace special chars
        let encoded = attempt
            .replace(' ', "%20")
            .replace('\'', "%27")
            .replace(';', "%3B")
            .replace('*', "%2A")
            .replace('-', "%2D");
        let response = app
            .client
            .get(format!("/api/v1/products?search={}", encoded))
            .header(bearer_header(&token))
            .dispatch();

        // Should return OK (empty or no results) not a server error
        assert_eq!(
            response.status(),
            Status::Ok,
            "Injection attempt should not cause error: {}",
            attempt
        );

        let body: PaginatedResponse<serde_json::Value> = response.into_json().unwrap();
        // Should return empty (no matches) not all data
        assert_eq!(body.data.len(), 0, "Injection should not return extra data");
    }
}

// ============================================
// Sensitive Data in Responses
// ============================================

#[test]
#[serial]
fn password_reset_token_not_in_user_response() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());

    // Set a reset token on the user
    use backend::schema::users;
    use diesel::prelude::*;

    diesel::update(users::table.find(admin.id))
        .set(users::password_reset_token.eq(Some("secret-reset-token")))
        .execute(&mut app.db_conn())
        .expect("Failed to set reset token");

    let token = app.access_token_for(admin.id, "admin");

    // Check /me
    let me_response = app
        .client
        .get("/api/v1/auth/me")
        .header(bearer_header(&token))
        .dispatch();

    let body: serde_json::Value = me_response.into_json().unwrap();
    assert!(body.get("password_reset_token").is_none());
    assert!(body.get("passwordResetToken").is_none());

    // Check /employees/:id
    let emp_response = app
        .client
        .get(format!("/api/v1/employees/{}", admin.id))
        .header(bearer_header(&token))
        .dispatch();

    let emp_body: serde_json::Value = emp_response.into_json().unwrap();
    assert!(emp_body.get("password_reset_token").is_none());
    assert!(emp_body.get("passwordResetToken").is_none());
}
