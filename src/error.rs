use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{self, Responder};
use rocket_dyn_templates::{Template, context};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    // Database errors
    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),
    #[error("Connection pool error: {0}")]
    Pool(#[from] diesel::r2d2::Error),

    // Auth errors
    #[error("Authentication required")]
    Unauthorized,
    #[error("Access denied")]
    Forbidden,

    // Resource errors
    #[error("Resource not found")]
    NotFound,

    // Validation errors
    #[error("{0}")]
    Validation(String),
}

impl AppError {
    /// Create a validation error with a custom message.
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// HTTP status code for errors.
    fn status(&self) -> Status {
        match self {
            Self::Database(_) | Self::Pool(_) => Status::InternalServerError,
            Self::Unauthorized => Status::Unauthorized,
            Self::Forbidden => Status::Forbidden,
            Self::NotFound => Status::NotFound,
            Self::Validation(_) => Status::BadRequest,
        }
    }

    /// User-safe message. Never expose internal details.
    fn user_message(&self) -> &str {
        match self {
            Self::Database(_) | Self::Pool(_) => {
                "An internal error occurred. Please try again later."
            }
            Self::Unauthorized => "Please log in to continue.",
            Self::Forbidden => "You don't have permission to access this resource.",
            Self::NotFound => "The requested resource was not found.",
            Self::Validation(msg) => msg,
        }
    }
}

impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static> {
        let status = self.status();

        // Log full error details server-side
        tracing::error!(
            error = %self, // use Display trait
            error_debug = ?self, // use Debug trait
            status = status.code,
            uri = %request.uri(),
            "Request failed"
        );

        // Render user-safe error page
        let ctx = context! {
            status_code: status.code,
            status_reason: status.reason().unwrap_or("Error"),
            message: self.user_message(),
        };

        Template::render("error", ctx)
            .respond_to(request)
            .map(|mut response| {
                response.set_status(status);
                response
            })
    }
}
