use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: Uuid,
    pub email: String,
    pub role: String,
    pub iat: i64, // Issued At
    pub exp: i64, // Expires At
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: Uuid,
    pub jti: Uuid,    // Unique token ID | JWT ID
    pub family: Uuid, // Token family
    pub iat: i64,
    pub exp: i64,
}
