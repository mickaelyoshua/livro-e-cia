use diesel::{prelude::*, PgTextExpressionMethods, QueryDsl, SelectableHelper};
use rocket::{get, post, serde::json::Json, State};
use shared::{AuthResponse, LoginRequest, UserDto};

use crate::{
    auth::{guards::AuthUser, jwt, password},
    db::pool::DbConnection,
    error::ApiError,
    models::{roles::Role, user::User},
    schema::{roles, users},
};

#[post("/api/v1/auth/login", format = "json", data = "<credentials>")]
pub async fn login(
    credentials: Json<LoginRequest>,
    mut db: DbConnection,
    jwt_secret: &State<String>,
) -> Result<Json<AuthResponse>, ApiError> {
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

    // Update last_login_at timestamp (ignore errors - non-critical)
    use chrono::Utc;
    diesel::update(users::table.find(user.id))
        .set(users::last_login_at.eq(Some(Utc::now())))
        .execute(&mut db.0)
        .ok();

    log::info!("Successful login: {}", email);

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user: user.into_dto(role), // exclude password_hash
    }))
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
