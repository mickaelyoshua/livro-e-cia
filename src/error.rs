use askama::Template;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    // Infrastructure
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("internal error: {0}")]
    Internal(String),

    // Auth
    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("token expired")]
    TokenExpired,

    #[error("token reuse detected")]
    TokenReuse,

    // Resource
    #[error("not found")]
    NotFound,

    #[error("validation error: {0}")]
    Validation(String),
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Database(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Unauthorized | Self::InvalidCredentials | Self::TokenExpired => {
                StatusCode::UNAUTHORIZED
            }
            Self::Forbidden | Self::TokenReuse => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
        }
    }

    fn user_message(&self) -> &str {
        match self {
            Self::Database(_) | Self::Internal(_) => {
                "Ocorreu um erro interno. Tente novamente mais tarde."
            }
            Self::Unauthorized => "Você precisa estar autenticado para acessar este recurso.",
            Self::Forbidden | Self::TokenReuse => {
                "Você não tem permissão para acessar este recurso."
            }
            Self::InvalidCredentials => "Email ou senha inválidos.",
            Self::TokenExpired => "Sua sessão expirou. Faça login novamente.",
            Self::NotFound => "Recurso não encontrado.",
            Self::Validation(msg) => msg,
        }
    }
}

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorTemplate {
    status: u16,
    reason: String,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();

        match &self {
            Self::NotFound => tracing::debug!(error = %self, "not found"),
            Self::Validation(_) => tracing::debug!(error = %self, "validation error"),
            Self::Unauthorized | Self::InvalidCredentials | Self::TokenExpired => {
                tracing::warn!(error = %self, "auth error")
            }
            _ => tracing::error!(error = %self, status = status.as_u16(), "request error"),
        }

        let template = ErrorTemplate {
            status: status.as_u16(),
            reason: status
                .canonical_reason()
                .unwrap_or("Error")
                .to_string(),
            message: self.user_message().to_string(),
        };

        match template.render() {
            Ok(html) => (status, axum::response::Html(html)).into_response(),
            Err(_) => (status, self.user_message().to_string()).into_response(),
        }
    }
}
