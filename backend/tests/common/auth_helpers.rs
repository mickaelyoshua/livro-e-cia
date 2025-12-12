// Authentication helpers for integration tests
//
// Provides utilities for creating Authorization headers, expired tokens, etc.

use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use rocket::http::Header as RocketHeader;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT claims structure (mirrors backend::auth::jwt::Claims)
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    id: String,
    role: String,
    exp: i64,
    iat: i64,
    jti: String,
    token_type: String,
}

/// Creates an Authorization header with Bearer token
pub fn bearer_header(token: &str) -> RocketHeader<'static> {
    RocketHeader::new("Authorization", format!("Bearer {}", token))
}

/// Creates an expired access token for testing token expiration
pub fn expired_access_token(user_id: Uuid, role: &str, secret: &str) -> String {
    let now = Utc::now();
    let claims = Claims {
        id: user_id.to_string(),
        role: role.to_string(),
        exp: (now - Duration::hours(1)).timestamp(), // Expired 1 hour ago
        iat: (now - Duration::hours(2)).timestamp(),
        jti: Uuid::new_v4().to_string(),
        token_type: "access".to_string(),
    };

    let header = Header::new(Algorithm::HS256);
    let key = EncodingKey::from_secret(secret.as_bytes());
    encode(&header, &claims, &key).expect("Failed to create expired token")
}

/// Creates an expired refresh token for testing
pub fn expired_refresh_token(user_id: Uuid, role: &str, secret: &str) -> String {
    let now = Utc::now();
    let claims = Claims {
        id: user_id.to_string(),
        role: role.to_string(),
        exp: (now - Duration::days(1)).timestamp(), // Expired 1 day ago
        iat: (now - Duration::days(8)).timestamp(),
        jti: Uuid::new_v4().to_string(),
        token_type: "refresh".to_string(),
    };

    let header = Header::new(Algorithm::HS256);
    let key = EncodingKey::from_secret(secret.as_bytes());
    encode(&header, &claims, &key).expect("Failed to create expired refresh token")
}

/// Creates a token signed with a different secret (invalid signature)
pub fn token_with_wrong_secret(user_id: Uuid, role: &str) -> String {
    let now = Utc::now();
    let claims = Claims {
        id: user_id.to_string(),
        role: role.to_string(),
        exp: (now + Duration::hours(1)).timestamp(),
        iat: now.timestamp(),
        jti: Uuid::new_v4().to_string(),
        token_type: "access".to_string(),
    };

    let header = Header::new(Algorithm::HS256);
    let wrong_secret = "completely-different-secret-key-32bytes!";
    let key = EncodingKey::from_secret(wrong_secret.as_bytes());
    encode(&header, &claims, &key).expect("Failed to create token with wrong secret")
}

/// Returns a malformed token string
pub fn malformed_token() -> &'static str {
    "not.a.valid.jwt.token"
}

/// Returns an empty token string
pub fn empty_token() -> &'static str {
    ""
}

/// Creates an Authorization header without "Bearer" prefix (invalid format)
pub fn invalid_auth_header(token: &str) -> RocketHeader<'static> {
    RocketHeader::new("Authorization", token.to_string())
}

/// Creates an Authorization header with wrong scheme
pub fn basic_auth_header(token: &str) -> RocketHeader<'static> {
    RocketHeader::new("Authorization", format!("Basic {}", token))
}
