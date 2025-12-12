// Integration tests for employee endpoints
//
// Tests: list, get, create, update, delete employees (Admin only)

mod common;

use common::{
    bearer_header, create_employee_user, create_inactive_user, create_verified_admin,
    get_employee_role, TestApp,
};
use rocket::http::{ContentType, Status};
use serial_test::serial;
use shared::{CreateEmployeeRequest, PaginatedResponse, UpdateEmployeeRequest, UserDto};
use uuid::Uuid;

// ============================================
// GET /api/v1/employees (List - Admin Only)
// ============================================

#[test]
#[serial]
fn list_employees_requires_admin() {
    let app = TestApp::new();
    app.reset_database();

    let employee = create_employee_user(&mut app.db_conn());
    let token = app.access_token_for(employee.id, "employee");

    let response = app
        .client
        .get("/api/v1/employees")
        .header(bearer_header(&token))
        .dispatch();

    assert_eq!(response.status(), Status::Forbidden);
}

#[test]
#[serial]
fn list_employees_without_auth_returns_401() {
    let app = TestApp::new();
    app.reset_database();

    let response = app.client.get("/api/v1/employees").dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
#[serial]
fn list_employees_returns_paginated_response() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let _employee1 = create_employee_user(&mut app.db_conn());
    let _employee2 = create_employee_user(&mut app.db_conn());

    let token = app.access_token_for(admin.id, "admin");

    let response = app
        .client
        .get("/api/v1/employees")
        .header(bearer_header(&token))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let paginated: PaginatedResponse<UserDto> =
        response.into_json().expect("Should parse paginated response");

    // Admin + 2 employees = 3 users
    assert_eq!(paginated.data.len(), 3);
    assert_eq!(paginated.page, 1);
    assert!(paginated.per_page > 0);
}

#[test]
#[serial]
fn list_employees_filters_by_active_status() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let _active = create_employee_user(&mut app.db_conn());
    let _inactive = create_inactive_user(&mut app.db_conn());

    let token = app.access_token_for(admin.id, "admin");

    // Filter for active only
    let response = app
        .client
        .get("/api/v1/employees?is_active=true")
        .header(bearer_header(&token))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let paginated: PaginatedResponse<UserDto> = response.into_json().unwrap();
    // All returned should be active
    for user in &paginated.data {
        assert!(user.is_active);
    }
}

#[test]
#[serial]
fn list_employees_excludes_password_hash() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let _employee = create_employee_user(&mut app.db_conn());

    let token = app.access_token_for(admin.id, "admin");

    let response = app
        .client
        .get("/api/v1/employees")
        .header(bearer_header(&token))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    // Parse as raw JSON to verify no password_hash field
    let body: serde_json::Value = response.into_json().unwrap();
    let employees = body["data"].as_array().unwrap();

    for emp in employees {
        assert!(
            emp.get("password_hash").is_none(),
            "password_hash should not be in response"
        );
        assert!(
            emp.get("passwordHash").is_none(),
            "passwordHash should not be in response"
        );
    }
}

// ============================================
// GET /api/v1/employees/<id> (Get Single - Admin Only)
// ============================================

#[test]
#[serial]
fn get_employee_requires_admin() {
    let app = TestApp::new();
    app.reset_database();

    let employee = create_employee_user(&mut app.db_conn());
    let token = app.access_token_for(employee.id, "employee");

    let response = app
        .client
        .get(format!("/api/v1/employees/{}", employee.id))
        .header(bearer_header(&token))
        .dispatch();

    assert_eq!(response.status(), Status::Forbidden);
}

#[test]
#[serial]
fn get_employee_returns_user() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let employee = create_employee_user(&mut app.db_conn());

    let token = app.access_token_for(admin.id, "admin");

    let response = app
        .client
        .get(format!("/api/v1/employees/{}", employee.id))
        .header(bearer_header(&token))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let user: UserDto = response.into_json().expect("Should parse UserDto");
    assert_eq!(user.id, employee.id);
    assert_eq!(user.email, employee.email);
}

#[test]
#[serial]
fn get_employee_not_found() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let token = app.access_token_for(admin.id, "admin");

    let response = app
        .client
        .get(format!("/api/v1/employees/{}", Uuid::new_v4()))
        .header(bearer_header(&token))
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);
}

// ============================================
// POST /api/v1/employees (Create - Admin Only)
// ============================================

#[test]
#[serial]
fn create_employee_requires_admin() {
    let app = TestApp::new();
    app.reset_database();

    let employee = create_employee_user(&mut app.db_conn());
    let role = get_employee_role(&mut app.db_conn());

    let token = app.access_token_for(employee.id, "employee");

    let create_request = CreateEmployeeRequest {
        email: "new@example.com".to_string(),
        name: "New Employee".to_string(),
        role_id: role.id,
    };

    let response = app
        .client
        .post("/api/v1/employees")
        .header(bearer_header(&token))
        .header(ContentType::JSON)
        .body(serde_json::to_string(&create_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Forbidden);
}

#[test]
#[serial]
fn create_employee_as_admin_succeeds() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let role = get_employee_role(&mut app.db_conn());

    let token = app.access_token_for(admin.id, "admin");

    let create_request = CreateEmployeeRequest {
        email: "new.employee@example.com".to_string(),
        name: "New Employee".to_string(),
        role_id: role.id,
    };

    let response = app
        .client
        .post("/api/v1/employees")
        .header(bearer_header(&token))
        .header(ContentType::JSON)
        .body(serde_json::to_string(&create_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Created);

    let user: UserDto = response.into_json().expect("Should parse UserDto");
    assert_eq!(user.email, "new.employee@example.com");
    assert_eq!(user.name, "New Employee");
    assert!(!user.email_verified); // New employees start unverified
}

#[test]
#[serial]
fn create_employee_duplicate_email_fails() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let existing = create_employee_user(&mut app.db_conn());
    let role = get_employee_role(&mut app.db_conn());

    let token = app.access_token_for(admin.id, "admin");

    let create_request = CreateEmployeeRequest {
        email: existing.email, // Duplicate!
        name: "Another Employee".to_string(),
        role_id: role.id,
    };

    let response = app
        .client
        .post("/api/v1/employees")
        .header(bearer_header(&token))
        .header(ContentType::JSON)
        .body(serde_json::to_string(&create_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
#[serial]
fn create_employee_invalid_role_fails() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let token = app.access_token_for(admin.id, "admin");

    let create_request = CreateEmployeeRequest {
        email: "new@example.com".to_string(),
        name: "New Employee".to_string(),
        role_id: Uuid::new_v4(), // Non-existent role
    };

    let response = app
        .client
        .post("/api/v1/employees")
        .header(bearer_header(&token))
        .header(ContentType::JSON)
        .body(serde_json::to_string(&create_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
#[serial]
fn create_employee_invalid_email_format_fails() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let role = get_employee_role(&mut app.db_conn());
    let token = app.access_token_for(admin.id, "admin");

    let create_request = serde_json::json!({
        "email": "not-an-email",
        "name": "New Employee",
        "role_id": role.id
    });

    let response = app
        .client
        .post("/api/v1/employees")
        .header(bearer_header(&token))
        .header(ContentType::JSON)
        .body(serde_json::to_string(&create_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}

// ============================================
// PUT /api/v1/employees/<id> (Update - Admin Only)
// ============================================

#[test]
#[serial]
fn update_employee_requires_admin() {
    let app = TestApp::new();
    app.reset_database();

    let employee = create_employee_user(&mut app.db_conn());
    let token = app.access_token_for(employee.id, "employee");

    let update_request = UpdateEmployeeRequest {
        name: Some("Updated Name".to_string()),
        role_id: None,
        is_active: None,
    };

    let response = app
        .client
        .put(format!("/api/v1/employees/{}", employee.id))
        .header(bearer_header(&token))
        .header(ContentType::JSON)
        .body(serde_json::to_string(&update_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Forbidden);
}

#[test]
#[serial]
fn update_employee_as_admin_succeeds() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let employee = create_employee_user(&mut app.db_conn());

    let token = app.access_token_for(admin.id, "admin");

    let update_request = UpdateEmployeeRequest {
        name: Some("Updated Name".to_string()),
        role_id: None,
        is_active: None,
    };

    let response = app
        .client
        .put(format!("/api/v1/employees/{}", employee.id))
        .header(bearer_header(&token))
        .header(ContentType::JSON)
        .body(serde_json::to_string(&update_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let updated: UserDto = response.into_json().expect("Should parse UserDto");
    assert_eq!(updated.name, "Updated Name");
}

#[test]
#[serial]
fn update_employee_can_change_role() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let employee = create_employee_user(&mut app.db_conn());
    let admin_role_id = app.admin_role_id();

    let token = app.access_token_for(admin.id, "admin");

    let update_request = UpdateEmployeeRequest {
        name: None,
        role_id: Some(admin_role_id),
        is_active: None,
    };

    let response = app
        .client
        .put(format!("/api/v1/employees/{}", employee.id))
        .header(bearer_header(&token))
        .header(ContentType::JSON)
        .body(serde_json::to_string(&update_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let updated: UserDto = response.into_json().expect("Should parse UserDto");
    assert_eq!(updated.role.name, "admin");
}

#[test]
#[serial]
fn update_employee_can_deactivate() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let employee = create_employee_user(&mut app.db_conn());

    let token = app.access_token_for(admin.id, "admin");

    let update_request = UpdateEmployeeRequest {
        name: None,
        role_id: None,
        is_active: Some(false),
    };

    let response = app
        .client
        .put(format!("/api/v1/employees/{}", employee.id))
        .header(bearer_header(&token))
        .header(ContentType::JSON)
        .body(serde_json::to_string(&update_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let updated: UserDto = response.into_json().expect("Should parse UserDto");
    assert!(!updated.is_active);
}

#[test]
#[serial]
fn update_employee_not_found() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let token = app.access_token_for(admin.id, "admin");

    let update_request = UpdateEmployeeRequest {
        name: Some("Updated Name".to_string()),
        role_id: None,
        is_active: None,
    };

    let response = app
        .client
        .put(format!("/api/v1/employees/{}", Uuid::new_v4()))
        .header(bearer_header(&token))
        .header(ContentType::JSON)
        .body(serde_json::to_string(&update_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);
}

// ============================================
// DELETE /api/v1/employees/<id> (Soft Delete - Admin Only)
// ============================================

#[test]
#[serial]
fn delete_employee_requires_admin() {
    let app = TestApp::new();
    app.reset_database();

    let employee = create_employee_user(&mut app.db_conn());
    let other_employee = create_employee_user(&mut app.db_conn());

    let token = app.access_token_for(employee.id, "employee");

    let response = app
        .client
        .delete(format!("/api/v1/employees/{}", other_employee.id))
        .header(bearer_header(&token))
        .dispatch();

    assert_eq!(response.status(), Status::Forbidden);
}

#[test]
#[serial]
fn delete_employee_soft_deletes() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let employee = create_employee_user(&mut app.db_conn());

    let token = app.access_token_for(admin.id, "admin");

    let response = app
        .client
        .delete(format!("/api/v1/employees/{}", employee.id))
        .header(bearer_header(&token))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    // Verify employee is soft-deleted in DB
    use backend::schema::users;
    use diesel::prelude::*;

    let deleted: backend::models::User = users::table
        .find(employee.id)
        .first(&mut app.db_conn())
        .expect("User should still exist in DB");

    assert!(!deleted.is_active);
}

#[test]
#[serial]
fn delete_employee_not_found() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let token = app.access_token_for(admin.id, "admin");

    let response = app
        .client
        .delete(format!("/api/v1/employees/{}", Uuid::new_v4()))
        .header(bearer_header(&token))
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);
}

#[test]
#[serial]
fn delete_employee_already_deleted_returns_404() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let inactive = create_inactive_user(&mut app.db_conn());

    let token = app.access_token_for(admin.id, "admin");

    let response = app
        .client
        .delete(format!("/api/v1/employees/{}", inactive.id))
        .header(bearer_header(&token))
        .dispatch();

    // Already inactive employees should return 404
    assert_eq!(response.status(), Status::NotFound);
}
