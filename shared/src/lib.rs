// Data Transfer Objects

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    let mut score = 0;
    if password.len() >= 12 {
        score += 1;
    }
    if password.chars().any(|c| c.is_lowercase()) {
        score += 1;
    }
    if password.chars().any(|c| c.is_uppercase()) {
        score += 1;
    }
    if password.chars().any(|c| c.is_numeric()) {
        score += 1;
    }
    if password.chars().any(|c| !c.is_alphanumeric()) {
        score += 1;
    }

    if score >= 3 {
        Ok(())
    } else {
        Err(ValidationError::new("weak_password"))
    }
}

// Custom validator for non-negative Decimal values
fn validate_non_negative_decimal(value: &Decimal) -> Result<(), ValidationError> {
    if value.is_sign_negative() {
        Err(ValidationError::new("negative_value"))
    } else {
        Ok(())
    }
}

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

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateProductRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(min = 1, max = 255))]
    pub author: String,
    // Validate and Decimal are not compatible
    // Validate needs the type to implement Copy and Decimal does not have it
    #[validate(custom(function = "validate_non_negative_decimal"))]
    pub price: Decimal,
    #[validate(range(min = 0))]
    pub stock_quantity: i32,
    #[validate(length(max = 255))]
    pub publisher: Option<String>,
    pub publication_date: Option<NaiveDate>,
    pub category_id: Uuid,
    #[validate(length(max = 5000))]
    pub description: Option<String>,
    #[validate(url, length(max = 500))]
    pub cover_image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateProductRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub author: Option<String>,
    #[validate(custom(function = "validate_non_negative_decimal"))]
    pub price: Option<Decimal>,
    #[validate(range(min = 0))]
    pub stock_quantity: Option<i32>,
    #[validate(length(max = 255))]
    pub publisher: Option<String>,
    pub publication_date: Option<NaiveDate>,
    pub category_id: Option<Uuid>,
    #[validate(length(max = 5000))]
    pub description: Option<String>,
    #[validate(url, length(max = 500))]
    pub cover_image_url: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email, length(max = 255))]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email, length(max = 255))]
    pub email: String,
    #[validate(
        length(min = 8, max = 128),
        custom(function = "validate_password_strength")
    )]
    pub password: String,
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub role_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Deserialize, Validate)]
pub struct ForgotPasswordRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    #[validate(length(min = 1))]
    pub token: String,
    #[validate(
        length(min = 8, max = 128, message = "Passowrd mut be at least 8 characters"),
        custom(function = "validate_password_strength")
    )]
    pub new_password: String,
}

// ========== Email Verification DTOs ==========

#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
}

// ========== Employee DTOs ==========

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateEmployeeRequest {
    #[validate(email, length(max = 255))]
    pub email: String,
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub role_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateEmployeeRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    pub role_id: Option<Uuid>,
    pub is_active: Option<bool>,
}
