// Custom error handling for JWT because there is many different cases
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("Failed to encode token: {0}")]
    EncodingError(#[from] jsonwebtoken::errors::Error),

    #[error("Token has expired")]
    Expired,

    #[error("Invalid token signature")]
    InvalidSignature,

    #[error("Invalid token format")]
    InvalidFormat,

    #[error("Missing required claim: {0}")]
    MissingClaim(String),
}

impl From<JwtError> for ApiError {
    fn from(err: JwtError) -> Self {
        match err {
            // Token validation errors should be Unauthorized
            JwtError::Expired
            | JwtError::InvalidSignature
            | JwtError::InvalidFormat
            | JwtError::MissingClaim(_) => ApiError::JwtError(err.to_string()),
            JwtError::EncodingError(_) => {
                log::error!("JWT encoding error: {:?}", err);
                ApiError::InternalError("Token generation failed".to_string())
            }
        }
    }
}

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: String,         // User ID
    pub role: String,       // User role
    pub exp: i64,           // Expiration
    pub iat: i64,           // Issued at
    pub jti: String,        // JWT ID (unique identifier)
    pub token_type: String, // "access" or "refresh"
}

const ACCESS_TOKEN_EXPIRY: i64 = 30; // 30 minutes
const REFRESH_TOKEN_EXPIRY: i64 = 7; // 7 days

pub fn generate_access_token(user_id: Uuid, role: &str, secret: &str) -> Result<String, JwtError> {
    let now = Utc::now();
    let expiration = now
        .checked_add_signed(Duration::minutes(ACCESS_TOKEN_EXPIRY))
        .ok_or(JwtError::InvalidFormat)?
        .timestamp();
    let claims = Claims {
        id: user_id.to_string(),
        role: role.to_string(),
        exp: expiration,
        iat: now.timestamp(),
        jti: Uuid::new_v4().to_string(),
        token_type: "access".to_string(),
    };

    let header = Header::new(Algorithm::HS256);
    let key = EncodingKey::from_secret(secret.as_bytes());
    encode(&header, &claims, &key).map_err(JwtError::from)
}

pub fn generate_refresh_token(user_id: Uuid, role: &str, secret: &str) -> Result<String, JwtError> {
    let now = Utc::now();
    let expiration = now
        .checked_add_signed(Duration::days(REFRESH_TOKEN_EXPIRY))
        .ok_or(JwtError::InvalidFormat)?
        .timestamp();
    let claims = Claims {
        id: user_id.to_string(),
        role: role.to_string(),
        exp: expiration,
        iat: now.timestamp(),
        jti: Uuid::new_v4().to_string(),
        token_type: "refresh".to_string(),
    };

    let header = Header::new(Algorithm::HS256);
    let key = EncodingKey::from_secret(secret.as_bytes());
    encode(&header, &claims, &key).map_err(JwtError::from)
}

pub fn validate_token(token: &str, secret: &str) -> Result<Claims, JwtError> {
    let key = DecodingKey::from_secret(secret.as_bytes());
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    let token_data = decode::<Claims>(token, &key, &validation).map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::Expired,
        jsonwebtoken::errors::ErrorKind::InvalidSignature => JwtError::InvalidSignature,
        _ => JwtError::InvalidFormat,
    })?;
    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &str = "test-secret-at-least-32-bytes-long!!";

    #[test]
    fn generate_access_token_creates_valid_jwt() {
        let user_id = Uuid::new_v4();
        let token = generate_access_token(user_id, "admin", TEST_SECRET)
            .expect("Should generate token");
        assert_eq!(token.split('.').count(), 3);
    }

    #[test]
    fn generate_access_token_includes_correct_claims() {
        let user_id = Uuid::new_v4();
        let token = generate_access_token(user_id, "employee", TEST_SECRET)
            .expect("Should generate token");
        let claims = validate_token(&token, TEST_SECRET)
            .expect("Should validate own token");

        assert_eq!(claims.id, user_id.to_string());
        assert_eq!(claims.role, "employee");
        assert_eq!(claims.token_type, "access");
    }

    #[test]
    fn generate_refresh_token_has_different_type() {
        let user_id = Uuid::new_v4();
        let token = generate_refresh_token(user_id, "admin", TEST_SECRET)
            .expect("Should generate token");
        let claims = validate_token(&token, TEST_SECRET).expect("Should validate");

        assert_eq!(claims.token_type, "refresh");
    }

    #[test]
    fn access_token_expires_after_30_minutes() {
        let user_id = Uuid::new_v4();
        let token = generate_access_token(user_id, "admin", TEST_SECRET)
            .expect("Should generate token");
        let claims = validate_token(&token, TEST_SECRET).expect("Should validate");

        let now = Utc::now().timestamp();
        let expected_expiry = now + (30 * 60);
        assert!((claims.exp - expected_expiry).abs() < 5);
    }

    #[test]
    fn refresh_token_expires_after_7_days() {
        let user_id = Uuid::new_v4();
        let token = generate_refresh_token(user_id, "admin", TEST_SECRET)
            .expect("Should generate token");
        let claims = validate_token(&token, TEST_SECRET).expect("Should validate");

        let now = Utc::now().timestamp();
        let expected_expiry = now + (7 * 24 * 60 * 60);
        assert!((claims.exp - expected_expiry).abs() < 5);
    }

    #[test]
    fn validate_token_rejects_wrong_secret() {
        let user_id = Uuid::new_v4();
        let token = generate_access_token(user_id, "admin", TEST_SECRET)
            .expect("Should generate");

        let result = validate_token(&token, "different-secret-32-bytes-long!!");
        assert!(result.is_err());
    }

    #[test]
    fn validate_token_rejects_malformed_token() {
        let result = validate_token("not.a.valid.jwt", TEST_SECRET);
        assert!(result.is_err());
    }

    #[test]
    fn validate_token_rejects_empty_token() {
        let result = validate_token("", TEST_SECRET);
        assert!(result.is_err());
    }

    #[test]
    fn handles_unicode_in_role() {
        let user_id = Uuid::new_v4();
        let role = "administrador";

        let token = generate_access_token(user_id, role, TEST_SECRET)
            .expect("Should generate");
        let claims = validate_token(&token, TEST_SECRET).expect("Should validate");

        assert_eq!(claims.role, role);
    }
}
