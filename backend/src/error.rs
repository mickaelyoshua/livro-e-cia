use std::io::Cursor;

use argon2::password_hash::Error as ArgonError;
use diesel::result::Error as DieselError;
use rocket::{
    http::{ContentType, Header, Status},
    response::{self, Responder, Response},
    serde::json::json,
    Request,
};
use thiserror::Error;

/// API error types with security-conscious error handling
///
/// Security principle: Internal errors are logged server-side but return
/// generic messages to client

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Not found: {0}")] // attibutes to define display messages
    NotFound(String),

    #[error("Database error")]
    DatabaseError(#[from] DieselError), // #[from] automaticaly generates From trait implementation

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Authentication failed: {0}")]
    Unauthorized(String),

    #[error("Access forbidden: {0}")]
    Forbidden(String),

    #[error("Internal server error")]
    InternalError(String),

    #[error("Password operation failed")]
    PasswordError(String),

    #[error("Invalid token: {0}")]
    JwtError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded { retry_after_seconds: u64 },
}

// Responder trait is Rocket's way of converting types into HTTP responses.
// When the route return a Result<T, ApiError> it will call 'respond_to'.
impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, _request: &'r Request<'_>) -> response::Result<'static> {
        // Extract retry_after before consuming self in match
        let retry_after = match &self {
            ApiError::RateLimitExceeded {
                retry_after_seconds,
            } => Some(*retry_after_seconds),
            _ => None,
        };

        let (status, message) = match &self {
            ApiError::NotFound(msg) => (Status::NotFound, msg.clone()),
            ApiError::DatabaseError(err) => {
                // SECURITY: log full error, return generic message
                log::error!("Database error: {:?}", err);
                (
                    Status::InternalServerError,
                    "Internal server error".to_string(),
                )
            }
            ApiError::ValidationError(msg) => (Status::BadRequest, msg.clone()),
            ApiError::Unauthorized(msg) => (Status::Unauthorized, msg.clone()),
            ApiError::Forbidden(msg) => (Status::Forbidden, msg.clone()),
            ApiError::InternalError(msg) => {
                // SECURITY: log full error, return generic message
                log::error!("Internal error: {}", msg);
                (
                    Status::InternalServerError,
                    "Internal server error".to_string(),
                )
            }
            ApiError::PasswordError(msg) => {
                log::error!("Password error: {}", msg);
                (
                    Status::InternalServerError,
                    "Internal server error".to_string(),
                )
            }
            ApiError::JwtError(msg) => {
                log::warn!("JWT error: {}", msg);
                (Status::Unauthorized, "Invalid or expired token".to_string())
            }
            ApiError::RateLimitExceeded { .. } => {
                (Status::TooManyRequests, "Too many requests. Please try again later.".to_string())
            }
        };

        let body = json!({
            "status": status.code,
            "error": message,
        })
        .to_string();

        let mut builder = Response::build();
        builder
            .status(status)
            .header(ContentType::JSON)
            // 'sized_body' needs a type that implement the Read trait, String does not.
            // Cursor will wrap the type and implement the Read trait
            .sized_body(body.len(), Cursor::new(body));

        if let Some(seconds) = retry_after {
            builder.header(Header::new("Retry-After", seconds.to_string()));
        }

        builder.ok()
    }
}

// #[from] not possible because argon2::password_hash::Error type doen't satisfy all the trait
// bounds that thiserror need
// Manually implementation then
impl From<ArgonError> for ApiError {
    fn from(value: ArgonError) -> Self {
        // SECURITY: Never expose password operation details
        ApiError::PasswordError(format!("Password error: {:?}", value))
    }
}
