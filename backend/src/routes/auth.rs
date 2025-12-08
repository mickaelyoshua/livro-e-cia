use chrono::{Duration, Utc};
use diesel::{prelude::*, PgTextExpressionMethods, QueryDsl, SelectableHelper};
use rocket::{get, post, serde::json::Json, State};
use serde_json::Value;
use shared::{AuthResponse, LoginRequest, LogoutRequest, RefreshRequest, TokenResponse, UserDto};

use crate::{
    auth::{guards::AuthUser, jwt, password, refresh},
    db::pool::DbConnection,
    error::ApiError,
    models::{roles::Role, user::User},
    schema::{roles, users},
    utils::validate_dto,
};

const REFRESH_TOKEN_EXPIRY: i64 = 7; // 7 days

#[post("/api/v1/auth/login", format = "json", data = "<credentials>")]
pub async fn login(
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
    diesel::update(users::table.find(user.id))
        .set(users::last_login_at.eq(Some(Utc::now())))
        .execute(&mut db.0)
        .ok();

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
