// Data Transfer Objects

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ========== Role DTOs ==========

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleDto {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

// ========== User DTOs ==========

/// Safe for user response - NO PASSWORD HASH
#[derive(Debug, Serialize, Deserialize)]
pub struct UserDto {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: RoleDto,
    pub is_active: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

/// Compact user info
#[derive(Debug, Serialize, Deserialize)]
pub struct UserSummaryDto {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role_name: String,
}

// ========== Refresh Token DTOs ==========

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

// ========== Auth Request DTOs ==========

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
    pub role_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token_response: TokenResponse,
    pub user: UserDto,
}

// ========== Password Reset DTOs ==========

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

// ========== Email Verification DTOs ==========

#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
}
