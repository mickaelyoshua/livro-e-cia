// Rocket's mechanism for extracting and validating data from HTTP requests
// Runs before route handler

use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};
use uuid::Uuid;

use crate::auth::jwt::validate_token;

pub struct AuthUser {
    pub user_id: Uuid,
    pub role: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // Extract Authorization header
        let auth_header = match req.headers().get_one("Authorization") {
            Some(h) => h,
            None => return Outcome::Error((Status::Unauthorized, ())),
        };

        // Check Bearer format
        if !auth_header.starts_with("Bearer ") {
            return Outcome::Error((Status::Unauthorized, ()));
        }
        let token = &auth_header[7..];

        // Get JWT secret from Rocket managed state
        let secret = match req.rocket().state::<String>() {
            Some(s) => s,
            None => return Outcome::Error((Status::InternalServerError, ())),
        };

        // Validate token
        let claims = match validate_token(token, secret) {
            Ok(c) => c,
            Err(_) => return Outcome::Error((Status::Unauthorized, ())),
        };

        // Ensure access token (not refresh)
        if claims.token_type != "access" {
            return Outcome::Error((Status::Unauthorized, ()));
        }

        //Parse user ID
        let user_id = match Uuid::parse_str(&claims.id) {
            Ok(id) => id,
            Err(_) => return Outcome::Error((Status::Unauthorized, ())),
        };

        Outcome::Success(AuthUser {
            user_id,
            role: claims.role,
        })
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
