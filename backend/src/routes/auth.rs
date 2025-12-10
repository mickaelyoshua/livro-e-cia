use chrono::{Duration, Utc};
use diesel::{prelude::*, PgTextExpressionMethods, QueryDsl, SelectableHelper};
use rocket::{get, post, serde::json::Json, State};
use serde_json::{json, Value};
use shared::{
    AuthResponse, ForgotPasswordRequest, LoginRequest, LogoutRequest, RefreshRequest,
    ResetPasswordRequest, TokenResponse, UserDto, VerifyEmailRequest,
};

use crate::{
    auth::{guards::AuthUser, hash_token, jwt, password, refresh},
    db::pool::DbConnection,
    email::EmailService,
    error::ApiError,
    models::{roles::Role, user::User, UpdateUser},
    rate_limit::{
        RateLimitedForgotPassword, RateLimitedLogin, RateLimitedResetPassword,
        RateLimitedVerifyEmail,
    },
    schema::{roles, users},
    utils::validate_dto,
};

const REFRESH_TOKEN_EXPIRY: i64 = 7; // 7 days
const PASSWORD_RESET_TOKEN_EXPIRY_HOURS: i64 = 1; // 1 hour for security

#[post("/api/v1/auth/login", format = "json", data = "<credentials>")]
pub async fn login(
    _rate_limit: RateLimitedLogin,
    credentials: Json<LoginRequest>,
    mut db: DbConnection,
    jwt_secret: &State<String>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Fields validation
    validate_dto(&*credentials)?;

    // Normalize email
    let email = credentials.email.trim().to_lowercase();

    // Query user with role
    let (user, role) = users::table
        .inner_join(roles::table.on(users::role_id.eq(roles::id)))
        .filter(users::email.ilike(&email))
        .select((User::as_select(), Role::as_select()))
        .first::<(User, Role)>(&mut db.0)
        .map_err(|_| {
            // SECURITY: Generic message = don't reveal user doesn't exist
            log::warn!("Login attempt for non-existing email: {}", email);
            ApiError::Unauthorized("Invalid credentials".to_string())
        })?;

    let password_valid = password::verify_password(&credentials.password, &user.password_hash)?;

    // SECURITY: Return SAME generic error as "user not found"
    if !password_valid {
        log::warn!("Invalid password for user: {}", email);
        return Err(ApiError::Unauthorized("Invalid credentials".to_string()));
    }

    if !user.is_active {
        log::warn!("Inactive user login attempt: {}", email);
        return Err(ApiError::Forbidden("Account is inactive".to_string()));
    }

    let access_token = jwt::generate_access_token(user.id, &role.name, jwt_secret)?;
    let refresh_token = jwt::generate_refresh_token(user.id, &role.name, jwt_secret)?;

    // Store refresh token in database
    let refresh_expires = Utc::now() + Duration::days(REFRESH_TOKEN_EXPIRY);
    refresh::store_refresh_token(user.id, &refresh_token, refresh_expires, &mut db.0)?;

    // Update last_login_at timestamp (ignore errors - non-critical)
    use chrono::Utc;
    if let Err(e) = diesel::update(users::table.find(user.id))
        .set(users::last_login_at.eq(Some(Utc::now())))
        .execute(&mut db.0)
    {
        log::warn!(
            "Failed to update 'last_login_at' for user {}: {}",
            user.id,
            e
        );
    }

    log::info!("Successful login: {}", email);

    Ok(Json(AuthResponse {
        token_response: TokenResponse {
            access_token,
            refresh_token,
        },
        user: user.into_dto(role), // exclude password_hash
    }))
}

#[post("/api/v1/auth/logout", format = "json", data = "<request>")]
pub async fn logout(
    request: Json<LogoutRequest>,
    mut db: DbConnection,
    jwt_secret: &State<String>,
) -> Result<Json<Value>, ApiError> {
    let (_user, db_token) =
        refresh::validate_refresh_token(&request.refresh_token, jwt_secret, &mut db.0)?;

    refresh::revoke_refresh_token(db_token.id, "user_logout", &mut db.0)?;

    log::info!("User logged out successfully");

    Ok(Json(
        serde_json::json!({"message": "Logged out successfully"}),
    ))
}

#[get("/api/v1/auth/me")]
pub async fn get_current_user(
    auth_user: AuthUser, // Guard validate token automatically
    mut db: DbConnection,
) -> Result<Json<UserDto>, ApiError> {
    // Query user with role
    let (user, role) = users::table
        .inner_join(roles::table.on(users::role_id.eq(roles::id)))
        .filter(users::id.eq(&auth_user.user_id))
        .select((User::as_select(), Role::as_select()))
        .first::<(User, Role)>(&mut db.0)
        .map_err(|_| {
            // SECURITY: Generic message = don't reveal user doesn't exist
            log::warn!(
                "User from valid token not found in DB: {}",
                auth_user.user_id
            );
            ApiError::Unauthorized("User not found".to_string())
        })?;

    Ok(Json(user.into_dto(role)))
}

#[post("/api/v1/auth/refresh", format = "json", data = "<request>")]
pub async fn refresh_token(
    request: Json<RefreshRequest>,
    mut db: DbConnection,
    jwt_secret: &State<String>,
) -> Result<Json<TokenResponse>, ApiError> {
    let (user, old_token) =
        refresh::validate_refresh_token(&request.refresh_token, jwt_secret, &mut db.0)?;

    let role = roles::table
        .find(user.role_id)
        .first::<Role>(&mut db.0)
        .map_err(|e| {
            log::error!("Role not found for user {}: {:?}", user.id, e);
            ApiError::InternalError("Failed to get user role".to_string())
        })?;

    let new_access_token = jwt::generate_access_token(user.id, &role.name, jwt_secret)?;
    let new_refresh_token = jwt::generate_refresh_token(user.id, &role.name, jwt_secret)?;

    let refresh_expires = Utc::now() + Duration::days(REFRESH_TOKEN_EXPIRY);
    refresh::store_refresh_token(user.id, &new_refresh_token, refresh_expires, &mut db.0)?;

    refresh::revoke_refresh_token(old_token.id, "used", &mut db.0)?;

    log::info!("Token refresh successful for user: {}", user.id);

    Ok(Json(TokenResponse {
        access_token: new_access_token,
        refresh_token: new_refresh_token,
    }))
}

#[post("/api/v1/auth/verify-email", format = "json", data = "<request>")]
pub async fn verify_email(
    _rate_limit: RateLimitedVerifyEmail,
    request: Json<VerifyEmailRequest>,
    mut db: DbConnection,
) -> Result<Json<Value>, ApiError> {
    let token_hash = hash_token(&request.token);

    // Find user with matching token
    let user = users::table
        .filter(users::password_reset_token.eq(&token_hash))
        .select(User::as_select())
        .first::<User>(&mut db.0)
        .optional()
        .map_err(|e| {
            log::error!("Database error: {}", e);
            ApiError::InternalError("Verification failed".to_string())
        })?;

    let user =
        user.ok_or_else(|| ApiError::ValidationError("Invalid or expired token".to_string()))?;

    if user.email_verified {
        return Ok(Json(json!({
            "message": "Email already verified",
            "email": user.email
        })));
    }

    if let Some(expires) = user.password_reset_expires_at {
        if expires < Utc::now() {
            return Err(ApiError::ValidationError("Token expired".to_string()));
        }
    }

    let update = UpdateUser {
        name: None,
        is_active: None,
        email_verified: Some(true),
        email_verified_at: Some(Some(Utc::now())),
        password_reset_token: Some(None),
        password_reset_expires_at: Some(None),
        last_login_at: None,
    };

    diesel::update(users::table.filter(users::id.eq(user.id)))
        .set(&update)
        .execute(&mut db.0)
        .map_err(|_| ApiError::InternalError("Verification failed".to_string()))?;

    log::info!("Email verified: {}", user.email);

    Ok(Json(json!({
        "message": "Email verified successfully",
        "email": user.email
    })))
}

#[post("/api/v1/auth/forgot-password", format = "json", data = "<request>")]
pub async fn forgot_password(
    _rate_limit: RateLimitedForgotPassword,
    request: Json<ForgotPasswordRequest>,
    mut db: DbConnection,
    email_service: &State<Box<dyn EmailService>>,
) -> Result<Json<Value>, ApiError> {
    validate_dto(&*request)?;

    let email = request.email.trim().to_lowercase();

    // Try to find user (don't reveal if exists)
    let user = users::table
        .filter(users::email.ilike(&email))
        .select(User::as_select())
        .first::<User>(&mut db.0)
        .optional()
        .map_err(|e| {
            log::error!("Database error in forgot_password {}", e);
            ApiError::InternalError("Request failed".to_string())
        })?;

    // If user exists and is active, send email
    if let Some(user) = user {
        if user.is_active {
            // Generate reset token (32 alphanumeric chars)
            use rand::Rng;
            let reset_token: String = rand::rng()
                .sample_iter(&rand::distr::Alphanumeric)
                .take(32)
                .map(char::from)
                .collect();

            // Hash token before storage
            let token_hash = hash_token(&reset_token);
            let token_expiry = Utc::now() + Duration::hours(PASSWORD_RESET_TOKEN_EXPIRY_HOURS);

            // Store hashed token
            let update = UpdateUser {
                name: None,
                is_active: None,
                email_verified: None,
                email_verified_at: None,
                password_reset_token: Some(Some(token_hash)),
                password_reset_expires_at: Some(Some(token_expiry)),
                last_login_at: None,
            };

            diesel::update(users::table.filter(users::id.eq(user.id)))
                .set(&update)
                .execute(&mut db.0)
                .map_err(|e| {
                    log::error!("Failed to store reset token for {}: {}", user.id, e);
                    ApiError::InternalError("Request failed".to_string())
                })?;

            // Send email with raw token (not hashed)
            if let Err(e) =
                email_service.send_password_reset_email(&user.email, &user.name, &reset_token)
            {
                log::error!(
                    "Failed to send password reset email to {}: {}",
                    user.email,
                    e
                );
            } else {
                log::info!("Password reset email sent to: {}", user.email);
            }
        } else {
            log::warn!("Password reset attempted for inactive user: {}", email);
        }
    } else {
        log::info!("Password reset request for non-existing email: {}", email);
    }

    // SECURITY: Always return same response (prevent email enumeration)
    Ok(Json(json!({
        "message": "If an account with that email exists, a password reset link has been sent."
    })))
}

#[post("/api/v1/auth/reset-password", format = "json", data = "<request>")]
pub async fn reset_password(
    _rate_limit: RateLimitedResetPassword,
    request: Json<ResetPasswordRequest>,
    mut db: DbConnection,
) -> Result<Json<Value>, ApiError> {
    validate_dto(&*request)?;

    let token_hash = hash_token(&request.token);

    // Find user with matching token
    let user = users::table
        .filter(users::password_reset_token.eq(&token_hash))
        .select(User::as_select())
        .first::<User>(&mut db.0)
        .optional()
        .map_err(|e| {
            log::error!("Database error in reset_password: {}", e);
            ApiError::InternalError("Password reset failed".to_string())
        })?;

    let user = user.ok_or_else(|| {
        log::warn!("Password reset attempted with invalid token");
        ApiError::ValidationError("Invalid or expired reset token".to_string())
    })?;

    // Check token expiration
    if let Some(expires) = user.password_reset_expires_at {
        if expires < Utc::now() {
            log::warn!(
                "Password reset attempted with expired token for user: {}",
                user.id
            );
            return Err(ApiError::ValidationError(
                "Reset token has expired".to_string(),
            ));
        }
    } else {
        return Err(ApiError::ValidationError(
            "Invalid or expired reset token".to_string(),
        ));
    }

    // Hash new password with Argon2id
    let new_password_hash = password::hash_password(&request.new_password).map_err(|e| {
        log::error!("Failed to hash new password: {}", e);
        ApiError::InternalError("Password reset failed".to_string())
    })?;

    // Update password and clear reset token
    diesel::update(users::table.filter(users::id.eq(user.id)))
        .set((
            users::password_hash.eq(new_password_hash),
            users::password_reset_token.eq::<Option<String>>(None),
            users::password_reset_expires_at.eq::<Option<chrono::DateTime<Utc>>>(None),
        ))
        .execute(&mut db.0)
        .map_err(|e| {
            log::error!("Failed to update password for user {}: {}", user.id, e);
            ApiError::InternalError("Password reset failed".to_string())
        })?;

    // SECURITY: Revoke all refresh tokens to force re-login everywhere
    if let Err(e) = refresh::revoke_all_user_tokens(user.id, "password_reset", &mut db.0) {
        log::error!(
            "Failed to revoke tokens after password reset for {}: {}",
            user.id,
            e
        );
    }

    log::info!(
        "Password reset successful for user: {} ({})",
        user.id,
        user.email
    );

    Ok(Json(json!({
        "message": "Password has been reset successfully. Please log in with your new password."
    })))
}
