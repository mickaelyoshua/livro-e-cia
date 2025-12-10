use crate::email::{EmailConfig, EmailService};

pub struct MockEmailService {
    config: EmailConfig,
}

impl MockEmailService {
    pub fn new(config: EmailConfig) -> Self {
        Self { config }
    }
}

impl EmailService for MockEmailService {
    fn send_verification_email(
        &self,
        to_email: &str,
        to_name: &str,
        token: &str,
    ) -> Result<(), super::EmailError> {
        let verification_url = format!("{}/verify-email?token={}", self.config.base_url, token);

        log::info!(
            "\n========== MOCK EMAIL ==========\n\
             To: {} <{}>\n\
             Subject: Verify your email - Livro e Cia\n\
             \n\
             Verification URL: {}\n\
             (Expires in 7 days)\n\
             =================================",
            to_name,
            to_email,
            verification_url
        );

        Ok(())
    }

    fn send_password_reset_email(
        &self,
        to_email: &str,
        to_name: &str,
        token: &str,
    ) -> Result<(), super::EmailError> {
        let reset_url = format!("{}/reset-password?token={}", self.config.base_url, token);

        log::info!(
            "\n========== MOCK EMAIL ==========\n\
             To: {} <{}>\n\
             Subject: Reset your password - Livro e Cia\n\
             \n\
             Password Reset URL: {}\n\
             (Expires in 1 hour)\n\
             =================================",
            to_name,
            to_email,
            reset_url
        );

        Ok(())
    }
}
