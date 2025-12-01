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
