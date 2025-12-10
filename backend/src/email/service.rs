use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("Failed to send email: {0}")]
    SendFailed(String),
}

pub struct EmailConfig {
    pub from_address: String,
    pub from_name: String,
    pub base_url: String,
}

impl EmailConfig {
    pub fn from_env() -> Self {
        use std::env;
        Self {
            from_address: env::var("EMAIL_FROM_ADDRESS")
                .unwrap_or_else(|_| "noreply@livroecia.local".to_string()),
            from_name: env::var("EMAIL_FROM_NAME").unwrap_or_else(|_| "Livro e Cia".to_string()),
            base_url: env::var("APP_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
        }
    }
}

pub trait EmailService: Send + Sync {
    fn send_verification_email(
        &self,
        to_email: &str,
        to_name: &str,
        token: &str,
    ) -> Result<(), EmailError>;

    fn send_password_reset_email(
        &self,
        to_email: &str,
        to_name: &str,
        token: &str,
    ) -> Result<(), EmailError>;
}
