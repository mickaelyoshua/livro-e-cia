use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode, errors, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: String,         // User ID
    pub role: String,       // User role
    pub exp: i64,           // Expiration
    pub iat: i64,           // Issued at
    pub token_type: String, // "access" or "refresh"
}

pub fn generate_access_token(
    user_id: Uuid,
    role: &str,
    secret: &str,
) -> Result<String, errors::Error> {
    let now = Utc::now();
    let claims = Claims {
        id: user_id.to_string(),
        role: role.to_string(),
        exp: (now + Duration::minutes(30)).timestamp(),
        iat: now.timestamp(),
        token_type: "access".to_string(),
    };

    let header = Header::new(Algorithm::HS256);
    let key = EncodingKey::from_secret(secret.as_bytes());
    encode(&header, &claims, &key)
}

pub fn generate_refresh_token(
    user_id: Uuid,
    role: &str,
    secret: &str,
) -> Result<String, errors::Error> {
    let now = Utc::now();
    let claims = Claims {
        id: user_id.to_string(),
        role: role.to_string(),
        exp: (now + Duration::days(7)).timestamp(),
        iat: now.timestamp(),
        token_type: "refresh".to_string(),
    };

    let header = Header::new(Algorithm::HS256);
    let key = EncodingKey::from_secret(secret.as_bytes());
    encode(&header, &claims, &key)
}

pub fn validate_token(token: &str, secret: &str) -> Result<Claims, errors::Error> {
    let key = DecodingKey::from_secret(secret.as_bytes());
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    let token_data = decode::<Claims>(token, &key, &validation)?;
    Ok(token_data.claims)
}
