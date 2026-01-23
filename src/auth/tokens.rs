use std::env;

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use crate::{
    auth::{AccessClaims, RefreshClaims},
    error::AppError,
};

const ACCESS_TOKEN_DURATION_SECS: i64 = 15 * 60; // 15 min
const REFRESH_TOKEN_DURATION_SECS: i64 = 7 * 24 * 3600; // 7 days

#[derive(Clone)]
pub struct JwtConfig {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtConfig {
    #[instrument]
    pub fn from_env() -> Result<Self, AppError> {
        let secret = env::var("JWT_SECRET")
            .map_err(|_| AppError::validation("JWT_SECRET environment variable must be set"))?;

        if secret.len() < 32 {
            warn!("JWT_SECRET is too short (minimum 32 bytes)");
            return Err(AppError::validation("JWT_SECRET must be at least 32 bytes"));
        }

        info!("JWT configuration loaded successfully");
        Ok(Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        })
    }

    #[instrument(skip(self), fields(employee_id = %employee_id, email = %email))]
    pub fn generate_access_token(
        &self,
        employee_id: Uuid,
        email: &str,
        role: &str,
    ) -> Result<String, AppError> {
        let now = chrono::Utc::now().timestamp();

        let claims = AccessClaims {
            sub: employee_id,
            email: email.to_string(),
            role: role.to_string(),
            iat: now,
            exp: now + ACCESS_TOKEN_DURATION_SECS,
        };

        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        debug!(role = %role, "Access token generated");
        Ok(token)
    }

    #[instrument(skip(self), fields(employee_id = %employee_id))]
    pub fn generate_refresh_token(
        &self,
        employee_id: Uuid,
    ) -> Result<(String, Uuid, Uuid), AppError> {
        // Return (token, family_id, jti)
        let now = chrono::Utc::now().timestamp();
        let family = Uuid::new_v4();
        let jti = Uuid::new_v4();

        let claims = RefreshClaims {
            sub: employee_id,
            jti,
            family,
            iat: now,
            exp: now + REFRESH_TOKEN_DURATION_SECS,
        };

        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        debug!(family = %family, jti = %jti, "Refresh token generated");
        Ok((token, family, jti))
    }

    #[instrument(skip(self, old_claims), fields(family = %old_claims.family, old_jti = %old_claims.jti))]
    pub fn rotate_refresh_token(
        &self,
        old_claims: &RefreshClaims,
    ) -> Result<(String, Uuid), AppError> {
        // Same family_id, new jti. Return (token, new_jti)
        let now = chrono::Utc::now().timestamp();
        let new_jti = Uuid::new_v4();

        let claims = RefreshClaims {
            sub: old_claims.sub,
            jti: new_jti,
            family: old_claims.family,
            iat: now,
            exp: now + REFRESH_TOKEN_DURATION_SECS,
        };

        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        debug!(new_jti = %new_jti, "Refresh token rotated");
        Ok((token, new_jti))
    }

    #[instrument(skip(self, token))]
    pub fn validate_access_token(&self, token: &str) -> Result<AccessClaims, AppError> {
        decode::<AccessClaims>(
            token,
            &self.decoding_key,
            &Validation::new(jsonwebtoken::Algorithm::HS256),
        )
        .map(|data| {
            debug!(employee_id = %data.claims.sub, "Access token validated");
            data.claims
        })
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                debug!("Access token expired");
                AppError::TokenExpired
            }
            _ => {
                warn!(error = %e, "Access token validation failed");
                AppError::Unauthorized
            }
        })
    }

    #[instrument(skip(self, token))]
    pub fn validate_refresh_token(&self, token: &str) -> Result<RefreshClaims, AppError> {
        decode::<RefreshClaims>(
            token,
            &self.decoding_key,
            &Validation::new(jsonwebtoken::Algorithm::HS256),
        )
        .map(|data| {
            debug!(employee_id = %data.claims.sub, family = %data.claims.family, "Refresh token validated");
            data.claims
        })
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                debug!("Refresh token expired");
                AppError::TokenExpired
            }
            _ => {
                warn!(error = %e, "Refresh token validation failed");
                AppError::Unauthorized
            }
        })
    }
}
