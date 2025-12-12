// Integration tests for authentication endpoints
//
// Tests: login, /me, refresh, logout, verify-email, forgot-password, reset-password

mod common;

use common::{
    bearer_header, create_inactive_user, create_unverified_user,
    create_user_with_expired_reset_token, create_user_with_reset_token, create_verified_admin,
    expired_access_token, hash_token, malformed_token, token_with_wrong_secret, TestApp,
    TEST_PASSWORD,
};
use rocket::http::{ContentType, Status};
use serial_test::serial;
use shared::{
    AuthResponse, LoginRequest, LogoutRequest, RefreshRequest, ResetPasswordRequest, TokenResponse,
    UserDto,
};

// ============================================
// POST /api/v1/auth/login
// ============================================

#[test]
#[serial]
fn login_valid_credentials_returns_tokens_and_user() {
    let app = TestApp::new();
    app.reset_database();

    let user = create_verified_admin(&mut app.db_conn());

    let login_request = LoginRequest {
        email: user.email.clone(),
        password: TEST_PASSWORD.to_string(),
    };

    let response = app
        .client
        .post("/api/v1/auth/login")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&login_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let auth_response: AuthResponse = response.into_json().expect("Should parse AuthResponse");

    assert!(!auth_response.token_response.access_token.is_empty());
    assert!(!auth_response.token_response.refresh_token.is_empty());
    assert_eq!(auth_response.user.email, user.email);
    assert_eq!(auth_response.user.role.name, "admin");
}

#[test]
#[serial]
fn login_normalizes_email_to_lowercase() {
    let app = TestApp::new();
    app.reset_database();

    let user = create_verified_admin(&mut app.db_conn());
    let uppercase_email = user.email.to_uppercase();

    let login_request = LoginRequest {
        email: uppercase_email,
        password: TEST_PASSWORD.to_string(),
    };

    let response = app
        .client
        .post("/api/v1/auth/login")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&login_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
}

#[test]
#[serial]
fn login_invalid_email_returns_401() {
    let app = TestApp::new();
    app.reset_database();

    let login_request = LoginRequest {
        email: "nonexistent@example.com".to_string(),
        password: TEST_PASSWORD.to_string(),
    };

    let response = app
        .client
        .post("/api/v1/auth/login")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&login_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);

    let body: serde_json::Value = response.into_json().expect("Should parse error");
    assert_eq!(body["error"], "Invalid credentials");
}

#[test]
#[serial]
fn login_wrong_password_returns_401_same_message() {
    let app = TestApp::new();
    app.reset_database();

    let user = create_verified_admin(&mut app.db_conn());

    let login_request = LoginRequest {
        email: user.email,
        password: "WrongPassword123!".to_string(),
    };

    let response = app
        .client
        .post("/api/v1/auth/login")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&login_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);

    // Same generic message as invalid email (prevents enumeration)
    let body: serde_json::Value = response.into_json().expect("Should parse error");
    assert_eq!(body["error"], "Invalid credentials");
}

#[test]
#[serial]
fn login_inactive_user_returns_403() {
    let app = TestApp::new();
    app.reset_database();

    let user = create_inactive_user(&mut app.db_conn());

    let login_request = LoginRequest {
        email: user.email,
        password: TEST_PASSWORD.to_string(),
    };

    let response = app
        .client
        .post("/api/v1/auth/login")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&login_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Forbidden);

    let body: serde_json::Value = response.into_json().expect("Should parse error");
    assert!(body["error"].as_str().unwrap().contains("inactive"));
}

#[test]
#[serial]
fn login_empty_email_returns_validation_error() {
    let app = TestApp::new();
    app.reset_database();

    let login_request = serde_json::json!({
        "email": "",
        "password": TEST_PASSWORD
    });

    let response = app
        .client
        .post("/api/v1/auth/login")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&login_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
#[serial]
fn login_empty_password_returns_validation_error() {
    let app = TestApp::new();
    app.reset_database();

    let login_request = serde_json::json!({
        "email": "test@example.com",
        "password": ""
    });

    let response = app
        .client
        .post("/api/v1/auth/login")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&login_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}

// ============================================
// GET /api/v1/auth/me
// ============================================

#[test]
#[serial]
fn me_valid_token_returns_user() {
    let app = TestApp::new();
    app.reset_database();

    let user = create_verified_admin(&mut app.db_conn());
    let token = app.access_token_for(user.id, "admin");

    let response = app
        .client
        .get("/api/v1/auth/me")
        .header(bearer_header(&token))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let user_dto: UserDto = response.into_json().expect("Should parse UserDto");
    assert_eq!(user_dto.id, user.id);
    assert_eq!(user_dto.email, user.email);
}

#[test]
#[serial]
fn me_missing_header_returns_401() {
    let app = TestApp::new();
    app.reset_database();

    let response = app.client.get("/api/v1/auth/me").dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
#[serial]
fn me_invalid_token_returns_401() {
    let app = TestApp::new();
    app.reset_database();

    let response = app
        .client
        .get("/api/v1/auth/me")
        .header(bearer_header(malformed_token()))
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
#[serial]
fn me_expired_token_returns_401() {
    let app = TestApp::new();
    app.reset_database();

    let user = create_verified_admin(&mut app.db_conn());
    let expired_token = expired_access_token(user.id, "admin", &app.jwt_secret);

    let response = app
        .client
        .get("/api/v1/auth/me")
        .header(bearer_header(&expired_token))
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
#[serial]
fn me_wrong_secret_returns_401() {
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
fn me_refresh_token_rejected() {
    let app = TestApp::new();
    app.reset_database();

    let user = create_verified_admin(&mut app.db_conn());
    // Use refresh token instead of access token
    let refresh_token = app.refresh_token_for(user.id, "admin");

    let response = app
        .client
        .get("/api/v1/auth/me")
        .header(bearer_header(&refresh_token))
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

// ============================================
// POST /api/v1/auth/refresh
// ============================================

#[test]
#[serial]
fn refresh_valid_returns_new_tokens() {
    let app = TestApp::new();
    app.reset_database();

    // First, login to get valid tokens
    let user = create_verified_admin(&mut app.db_conn());
    let login_request = LoginRequest {
        email: user.email,
        password: TEST_PASSWORD.to_string(),
    };

    let login_response = app
        .client
        .post("/api/v1/auth/login")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&login_request).unwrap())
        .dispatch();

    let auth: AuthResponse = login_response.into_json().unwrap();
    let refresh_token = auth.token_response.refresh_token;

    // Now refresh
    let refresh_request = RefreshRequest {
        refresh_token: refresh_token.clone(),
    };

    let response = app
        .client
        .post("/api/v1/auth/refresh")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&refresh_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let token_response: TokenResponse = response.into_json().expect("Should parse TokenResponse");
    assert!(!token_response.access_token.is_empty());
    assert!(!token_response.refresh_token.is_empty());
    // New refresh token should be different from old one
    assert_ne!(token_response.refresh_token, refresh_token);
}

#[test]
#[serial]
fn refresh_with_access_token_fails() {
    let app = TestApp::new();
    app.reset_database();

    let user = create_verified_admin(&mut app.db_conn());
    // Use access token instead of refresh token
    let access_token = app.access_token_for(user.id, "admin");

    let refresh_request = RefreshRequest {
        refresh_token: access_token,
    };

    let response = app
        .client
        .post("/api/v1/auth/refresh")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&refresh_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
#[serial]
fn refresh_with_invalid_token_fails() {
    let app = TestApp::new();
    app.reset_database();

    let refresh_request = RefreshRequest {
        refresh_token: "invalid-token".to_string(),
    };

    let response = app
        .client
        .post("/api/v1/auth/refresh")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&refresh_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

// ============================================
// POST /api/v1/auth/logout
// ============================================

#[test]
#[serial]
fn logout_revokes_token() {
    let app = TestApp::new();
    app.reset_database();

    // Login first
    let user = create_verified_admin(&mut app.db_conn());
    let login_request = LoginRequest {
        email: user.email,
        password: TEST_PASSWORD.to_string(),
    };

    let login_response = app
        .client
        .post("/api/v1/auth/login")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&login_request).unwrap())
        .dispatch();

    let auth: AuthResponse = login_response.into_json().unwrap();
    let refresh_token = auth.token_response.refresh_token;

    // Logout
    let logout_request = LogoutRequest {
        refresh_token: refresh_token.clone(),
    };

    let response = app
        .client
        .post("/api/v1/auth/logout")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&logout_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    // Try to use the revoked refresh token
    let refresh_request = RefreshRequest { refresh_token };

    let refresh_response = app
        .client
        .post("/api/v1/auth/refresh")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&refresh_request).unwrap())
        .dispatch();

    // Should fail because token was revoked
    assert_eq!(refresh_response.status(), Status::Unauthorized);
}

#[test]
#[serial]
fn logout_invalid_token_fails() {
    let app = TestApp::new();
    app.reset_database();

    let logout_request = LogoutRequest {
        refresh_token: "invalid-token".to_string(),
    };

    let response = app
        .client
        .post("/api/v1/auth/logout")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&logout_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

// ============================================
// POST /api/v1/auth/forgot-password
// ============================================

#[test]
#[serial]
fn forgot_password_always_returns_same_response() {
    let app = TestApp::new();
    app.reset_database();

    // Request for existing user
    let user = create_verified_admin(&mut app.db_conn());
    let request1 = serde_json::json!({ "email": user.email });

    let response1 = app
        .client
        .post("/api/v1/auth/forgot-password")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&request1).unwrap())
        .dispatch();

    assert_eq!(response1.status(), Status::Ok);
    let body1: serde_json::Value = response1.into_json().unwrap();

    // Request for non-existing user
    let request2 = serde_json::json!({ "email": "nonexistent@example.com" });

    let response2 = app
        .client
        .post("/api/v1/auth/forgot-password")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&request2).unwrap())
        .dispatch();

    assert_eq!(response2.status(), Status::Ok);
    let body2: serde_json::Value = response2.into_json().unwrap();

    // Both responses should have the same message (no email enumeration)
    assert_eq!(body1["message"], body2["message"]);
}

#[test]
#[serial]
fn forgot_password_invalid_email_format_fails() {
    let app = TestApp::new();
    app.reset_database();

    let request = serde_json::json!({ "email": "not-an-email" });

    let response = app
        .client
        .post("/api/v1/auth/forgot-password")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}

// ============================================
// POST /api/v1/auth/reset-password
// ============================================

#[test]
#[serial]
fn reset_password_valid_token_succeeds() {
    let app = TestApp::new();
    app.reset_database();

    let raw_token = "test-reset-token-12345678901234";
    let token_hash = hash_token(raw_token);
    let user = create_user_with_reset_token(&mut app.db_conn(), &token_hash);

    let reset_request = ResetPasswordRequest {
        token: raw_token.to_string(),
        new_password: "NewSecurePassword123!".to_string(),
    };

    let response = app
        .client
        .post("/api/v1/auth/reset-password")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&reset_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    // Verify can login with new password
    let login_request = LoginRequest {
        email: user.email,
        password: "NewSecurePassword123!".to_string(),
    };

    let login_response = app
        .client
        .post("/api/v1/auth/login")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&login_request).unwrap())
        .dispatch();

    assert_eq!(login_response.status(), Status::Ok);
}

#[test]
#[serial]
fn reset_password_invalid_token_fails() {
    let app = TestApp::new();
    app.reset_database();

    let reset_request = ResetPasswordRequest {
        token: "invalid-token".to_string(),
        new_password: "NewSecurePassword123!".to_string(),
    };

    let response = app
        .client
        .post("/api/v1/auth/reset-password")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&reset_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
#[serial]
fn reset_password_expired_token_fails() {
    let app = TestApp::new();
    app.reset_database();

    let raw_token = "expired-reset-token-1234567890";
    let token_hash = hash_token(raw_token);
    let _user = create_user_with_expired_reset_token(&mut app.db_conn(), &token_hash);

    let reset_request = ResetPasswordRequest {
        token: raw_token.to_string(),
        new_password: "NewSecurePassword123!".to_string(),
    };

    let response = app
        .client
        .post("/api/v1/auth/reset-password")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&reset_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
#[serial]
fn reset_password_weak_password_fails() {
    let app = TestApp::new();
    app.reset_database();

    let raw_token = "test-reset-token-weak-password";
    let token_hash = hash_token(raw_token);
    let _user = create_user_with_reset_token(&mut app.db_conn(), &token_hash);

    let reset_request = ResetPasswordRequest {
        token: raw_token.to_string(),
        new_password: "weak".to_string(), // Too short and simple
    };

    let response = app
        .client
        .post("/api/v1/auth/reset-password")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&reset_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}

// ============================================
// POST /api/v1/auth/verify-email
// ============================================

#[test]
#[serial]
fn verify_email_valid_token_succeeds() {
    let app = TestApp::new();
    app.reset_database();

    // Create unverified user with verification token
    let raw_token = "verify-email-token-123456789012";
    let token_hash = hash_token(raw_token);
    let user = create_unverified_user(&mut app.db_conn());

    // Set the verification token (stored in password_reset_token field)
    use backend::schema::users;
    use chrono::{Duration, Utc};
    use diesel::prelude::*;

    diesel::update(users::table.find(user.id))
        .set((
            users::password_reset_token.eq(Some(&token_hash)),
            users::password_reset_expires_at.eq(Some(Utc::now() + Duration::days(7))),
        ))
        .execute(&mut app.db_conn())
        .expect("Failed to set verification token");

    let verify_request = serde_json::json!({ "token": raw_token });

    let response = app
        .client
        .post("/api/v1/auth/verify-email")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&verify_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    // Verify the user is now verified in DB
    let updated_user: backend::models::User = users::table
        .find(user.id)
        .first(&mut app.db_conn())
        .expect("User should exist");

    assert!(updated_user.email_verified);
}

#[test]
#[serial]
fn verify_email_invalid_token_fails() {
    let app = TestApp::new();
    app.reset_database();

    let verify_request = serde_json::json!({ "token": "invalid-token" });

    let response = app
        .client
        .post("/api/v1/auth/verify-email")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&verify_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}
