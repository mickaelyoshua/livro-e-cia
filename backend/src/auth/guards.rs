// Rocket's mechanism for extracting and validating data from HTTP requests
// Runs before route handler

use diesel::prelude::*;
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request, State,
};
use uuid::Uuid;

use crate::{auth::jwt::validate_token, db::DbConnection, error::ApiError, schema::users};

pub struct AuthUser {
    pub user_id: Uuid,
    pub role: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = ApiError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // Get JWT secret from Rocket managed state
        let secret = match req.guard::<&State<String>>().await {
            Outcome::Success(secret) => secret.as_str(),
            _ => {
                log::error!("JWT secret not found in Rocket state");
                return Outcome::Error((
                    Status::InternalServerError,
                    ApiError::InternalError("Server misconfiguration".to_string()),
                ));
            }
        };

        // Extract token from Authorization header
        let token = match req.headers().get_one("Authorization") {
            Some(auth_header) => {
                // Check correct header
                if let Some(token) = auth_header.strip_prefix("Bearer ") {
                    token
                } else {
                    log::warn!("Invalid Authorization header format");
                    return Outcome::Error((
                        Status::Unauthorized,
                        ApiError::Unauthorized("Invalid authorization header".to_string()),
                    ));
                }
            }
            None => {
                log::warn!("Missing Authorization header");
                return Outcome::Error((
                    Status::Unauthorized,
                    ApiError::Unauthorized("Missing authorization token".to_string()),
                ));
            }
        };

        match validate_token(token, secret) {
            Ok(claims) => {
                // Verify token type
                if claims.token_type != "access" {
                    log::warn!("Invalid token type {}", claims.token_type);
                    return Outcome::Error((
                        Status::Unauthorized,
                        ApiError::Unauthorized("Invalid token type".to_string()),
                    ));
                }

                let user_id = match Uuid::parse_str(&claims.id) {
                    Ok(id) => id,
                    Err(_) => {
                        return Outcome::Error((
                            Status::Unauthorized,
                            ApiError::Unauthorized("Invalid user id".to_string()),
                        ))
                    }
                };

                // Verify user still exists and is active
                let mut db = match req.guard::<DbConnection>().await {
                    Outcome::Success(db) => db,
                    _ => {
                        log::error!("Database connection unavailable in AuthUser guard");
                        return Outcome::Error((
                            Status::InternalServerError,
                            ApiError::InternalError("Service unavailable".to_string()),
                        ));
                    }
                };

                let is_active: bool = match users::table
                    .find(user_id)
                    .select(users::is_active)
                    .first::<bool>(&mut db.0)
                    .optional()
                {
                    Ok(Some(active)) => active,
                    Ok(None) => {
                        log::warn!("Token for non-existent user: {}", user_id);
                        return Outcome::Error((
                            Status::Unauthorized,
                            ApiError::Unauthorized("Invalid token".to_string()),
                        ));
                    }
                    Err(e) => {
                        log::error!("Database error checking user status: {}", e);
                        return Outcome::Error((
                            Status::InternalServerError,
                            ApiError::InternalError("Service unavailable".to_string()),
                        ));
                    }
                };

                if !is_active {
                    log::warn!("Token for inactive user: {}", user_id);
                    return Outcome::Error((
                        Status::Unauthorized,
                        ApiError::Unauthorized("Account is inactive".to_string()),
                    ));
                }

                Outcome::Success(AuthUser {
                    user_id,
                    role: claims.role,
                })
            }
            Err(jwt_err) => Outcome::Error((Status::Unauthorized, jwt_err.into())),
        }
    }
}

pub struct OptionalAuthUser(pub Option<AuthUser>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for OptionalAuthUser {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.guard::<AuthUser>().await {
            Outcome::Success(user) => Outcome::Success(OptionalAuthUser(Some(user))),
            _ => Outcome::Success(OptionalAuthUser(None)),
        }
    }
}

pub struct AdminGuard(pub AuthUser);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminGuard {
    type Error = ApiError;
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth_user = match req.guard::<AuthUser>().await {
            Outcome::Success(user) => user,
            Outcome::Error((status, err)) => return Outcome::Error((status, err)),
            Outcome::Forward(status) => return Outcome::Forward(status),
        };

        if auth_user.role.to_lowercase() == "admin" {
            Outcome::Success(AdminGuard(auth_user))
        } else {
            log::warn!(
                "Non-admin user {} attempted admin action",
                auth_user.user_id
            );
            Outcome::Error((
                Status::Forbidden,
                ApiError::Forbidden("Admin access required".to_string()),
            ))
        }
    }
}
