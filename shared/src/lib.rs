// Data Transfer Objects

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
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

// ========== Category DTOs ==========

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryDto {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

// ========== Product DTOs ==========

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductDto {
    pub id: Uuid,
    pub title: String,
    pub author: String,
    pub price: Decimal,
    pub stock_quantity: i32,
    pub publisher: Option<String>,
    pub publication_date: Option<NaiveDate>,
    pub category: CategoryDto,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub title: String,
    pub author: String,
    pub price: Decimal,
    pub stock_quantity: i32,
    pub publisher: Option<String>,
    pub publication_date: Option<NaiveDate>,
    pub category_id: Uuid,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub title: Option<String>,
    pub author: Option<String>,
    pub price: Option<Decimal>,
    pub stock_quantity: Option<i32>,
    pub publisher: Option<String>,
    pub publication_date: Option<NaiveDate>,
    pub category_id: Option<Uuid>,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub page: i64,
    pub per_page: i64,
    pub total_count: i64,
    pub total_pages: i64,
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

// ========== Logout DTOs ==========

#[derive(Debug, Serialize, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
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
