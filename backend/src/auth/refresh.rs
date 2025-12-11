use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::PgConnection;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    auth::jwt,
    error::ApiError,
    models::{NewRefreshToken, RefreshToken, User},
    schema::{refresh_tokens, users},
};

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn store_refresh_token(
    user_id: Uuid,
    token: &str,
    expires_at: DateTime<Utc>,
    db: &mut PgConnection,
) -> Result<Uuid, ApiError> {
    let token_hash = hash_token(token);
    let new_token = NewRefreshToken {
        user_id,
        token_hash,
        expires_at,
    };

    diesel::insert_into(refresh_tokens::table)
        .values(&new_token)
        .returning(refresh_tokens::id)
        .get_result::<Uuid>(db)
        .map_err(|e| {
            log::error!("Failed to store refresh token: {:?}", e);
            ApiError::InternalError("Failed to store token".to_string())
        })
}

pub fn validate_refresh_token(
    token: &str,
    secret: &str,
    db: &mut PgConnection,
) -> Result<(User, RefreshToken), ApiError> {
    // Validate JWT signature and expiration
    let claims = jwt::validate_token(token, secret)?;

    if claims.token_type != "refresh" {
        log::warn!("Invalid token type for refresh: {}", claims.token_type);
        return Err(ApiError::Unauthorized("Invalid token type".to_string()));
    }

    let user_id = Uuid::parse_str(&claims.id).map_err(|_| {
        log::error!("Invalid user ID in token claims: {}", claims.id);
        ApiError::Unauthorized("Invalid token".to_string())
    })?;

    let token_hash = hash_token(token);
    let db_token = refresh_tokens::table
        .filter(refresh_tokens::token_hash.eq(&token_hash))
        .select(RefreshToken::as_select())
        .first::<RefreshToken>(db)
        .map_err(|_| {
            log::warn!("Refresh token not found in database: {}", token_hash);
            ApiError::Unauthorized("Invalid token".to_string())
        })?;

    if db_token.is_revoked() {
        log::error!(
            "SECURITY: Revoked refresh token attempted reuse!\nUser: {}\nToken: {}",
            user_id,
            db_token.id
        );
        return Err(ApiError::Unauthorized("Invalid token".to_string()));
    }

    if !db_token.is_valid() {
        log::warn!("Expired refresh token attempted use: {}", db_token.id);
        return Err(ApiError::Unauthorized("Token expired".to_string()));
    }

    let user = users::table
        .find(user_id)
        .select(User::as_select())
        .first::<User>(db)
        .map_err(|_| {
            log::warn!("User from valid token not found: {}", user_id);
            ApiError::Unauthorized("Invalid token".to_string())
        })?;

    if !user.is_active {
        log::warn!("Inactive user attempted token refresh: {}", user_id);
        return Err(ApiError::Forbidden("Account is inactive".to_string()));
    }

    diesel::update(refresh_tokens::table.find(db_token.id))
        .set(refresh_tokens::last_used_at.eq(Some(Utc::now())))
        .execute(db)
        .ok();

    Ok((user, db_token))
}

pub fn revoke_refresh_token(
    token_id: Uuid,
    reason: &str,
    db: &mut PgConnection,
) -> Result<(), ApiError> {
    diesel::update(refresh_tokens::table.find(token_id))
        .set((refresh_tokens::revoked_at.eq(Some(Utc::now())),))
        .execute(db)
        .map_err(|e| {
            log::error!("Failed to revoke token {}: {:?}", token_id, e);
            ApiError::InternalError("Failed to revoke token".to_string())
        })?;

    log::info!("Revoked refresh token: {}\nReason: {}", token_id, reason);
    Ok(())
}

pub fn revoke_all_user_tokens(
    user_id: Uuid,
    reason: &str,
    db: &mut PgConnection,
) -> Result<usize, ApiError> {
    let count = diesel::update(
        refresh_tokens::table
            .filter(refresh_tokens::user_id.eq(user_id))
            .filter(refresh_tokens::revoked_at.is_null()),
    )
    .set(refresh_tokens::revoked_at.eq(Some(Utc::now())))
    .execute(db)
    .map_err(|e| {
        log::error!("Failed to revoke user tokens {}: {:?}", user_id, e);
        ApiError::InternalError("Failed to revoke tokens".to_string())
    })?;

    log::info!(
        "Revoked {} tokens for user {}\nReason: {}",
        count,
        user_id,
        reason
    );
    Ok(count)
}

pub fn cleanup_expired_tokens(db: &mut PgConnection) -> Result<usize, ApiError> {
    let now = Utc::now();
    let count = diesel::delete(refresh_tokens::table.filter(refresh_tokens::expires_at.lt(now)))
        .execute(db)
        .map_err(|e| {
            log::error!("Failed to cleanup expired tokens: {:?}", e);
            ApiError::InternalError("Cleanup failed".to_string())
        })?;

    if count > 0 {
        log::info!("Cleaned up {} expired refresh tokens", count);
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_token_produces_sha256_hex() {
        let hash = hash_token("test-token-123");
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn hash_token_is_deterministic() {
        let hash1 = hash_token("same-token");
        let hash2 = hash_token("same-token");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn hash_token_produces_different_hashes_for_different_tokens() {
        let hash1 = hash_token("token1");
        let hash2 = hash_token("token2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn hash_token_is_case_sensitive() {
        let hash_lower = hash_token("token");
        let hash_upper = hash_token("TOKEN");
        assert_ne!(hash_lower, hash_upper);
    }

    #[test]
    fn hash_token_handles_empty_string() {
        let hash = hash_token("");
        assert_eq!(hash, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }
}
