// Rocket's mechanism for extracting and validating data from HTTP requests
// Runs before route handler

use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request, State,
};
use uuid::Uuid;

use crate::{auth::jwt::validate_token, error::ApiError};

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
