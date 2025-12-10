use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};

use crate::email::{EmailConfig, EmailError, EmailService};

pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
}

impl SmtpConfig {
    pub fn from_env() -> Result<Self, String> {
        use std::env;

        Ok(Self {
            host: env::var("SMTP_HOST").map_err(|_| "SMTP_HOST environment variable not set")?,
            port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".to_string())
                .parse()
                .map_err(|_| "Invalid SMTP_PORT")?,
            username: env::var("SMTP_USERNAME")
                .map_err(|_| "SMTP_USERNAME environment variable not set")?,
            password: env::var("SMTP_PASSWORD")
                .map_err(|_| "SMTP_PASSWORD environment variable not set")?,
            use_tls: env::var("SMTP_USE_TLS") // secure by default
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        })
    }
}

pub struct SmtpEmailService {
    config: EmailConfig,
    mailer: SmtpTransport, // internal connection pool for mailing
}

impl SmtpEmailService {
    pub fn new(email_config: EmailConfig, smtp_config: SmtpConfig) -> Result<Self, EmailError> {
        let creds = Credentials::new(smtp_config.username, smtp_config.password);

        let mailer = if smtp_config.use_tls {
            SmtpTransport::starttls_relay(&smtp_config.host) // connection to port 587 upgrated
                // with TLS encryption
                .map_err(|e| EmailError::SendFailed(format!("Failed to creat SMTP relay: {}", e)))?
                .port(smtp_config.port)
                .credentials(creds)
                .build()
        } else {
            // For local development without TLS (e.g., MailHog, Mailcatcher)
            SmtpTransport::builder_dangerous(&smtp_config.host)
                .port(smtp_config.port)
                .credentials(creds)
                .build()
        };

        Ok(Self {
            config: email_config,
            mailer,
        })
    }

    fn build_email(
        &self,
        to_email: &str,
        to_name: &str,
        subject: &str,
        body: String,
    ) -> Result<Message, EmailError> {
        let from_mailbox: Mailbox =
            format!("{} <{}>", self.config.from_name, self.config.from_address) // "Display name <email@example.com>"
                .parse()
                .map_err(|e| EmailError::SendFailed(format!("Invalid from address: {}", e)))?;

        let to_mailbox: Mailbox = format!("{} <{}>", to_name, to_email)
            .parse()
            .map_err(|e| EmailError::SendFailed(format!("Invalid from address: {}", e)))?;

        Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body)
            .map_err(|e| EmailError::SendFailed(format!("Failed to build email: {}", e)))
    }

    fn send(&self, message: Message) -> Result<(), EmailError> {
        self.mailer
            .send(&message)
            .map_err(|e| EmailError::SendFailed(format!("SMTP error: {}", e)))?;

        Ok(())
    }
}

impl EmailService for SmtpEmailService {
    fn send_verification_email(
        &self,
        to_email: &str,
        to_name: &str,
        token: &str,
    ) -> Result<(), EmailError> {
        let verification_url = format!("{}/verify-email?token={}", self.config.base_url, token);

        let body = format!(
            //   r#"..."# - Raw string literal
            r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
    <h2>Welcome to Livro e Cia!</h2>
    <p>Hi {name},</p>
    <p>Your account has been created. Please verify your email by clicking the link below:</p>
    <p><a href="{url}" style="display: inline-block; padding: 10px 20px; background-color: #4CAF50; color: white; text-decoration: none; border-radius: 5px;">Verify Email</a></p>
    <p>Or copy this link: {url}</p>
    <p>This link expires in 7 days.</p>
    <br>
    <p>Best regards,<br>Livro e Cia Team</p>
</body>
</html>"#,
            name = to_name,
            url = verification_url
        );

        let message =
            self.build_email(to_email, to_name, "Verify your email - Livro e Cia", body)?;
        self.send(message)?;
        log::info!("Verification email sent to: {}", to_email);
        Ok(())
    }

    fn send_password_reset_email(
        &self,
        to_email: &str,
        to_name: &str,
        token: &str,
    ) -> Result<(), EmailError> {
        let reset_url = format!("{}/reset-password?token={}", self.config.base_url, token);

        let body = format!(
            r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
    <h2>Password Reset Request</h2>
    <p>Hi {name},</p>
    <p>We received a request to reset your password. Click the link below to set a new password:</p>
    <p><a href="{url}" style="display: inline-block; padding: 10px 20px; background-color: #2196F3; color: white; text-decoration: none; border-radius: 5px;">Reset Password</a></p>
    <p>Or copy this link: {url}</p>
    <p><strong>This link expires in 1 hour.</strong></p>
    <p>If you didn't request this, please ignore this email. Your password will remain unchanged.</p>
    <br>
    <p>Best regards,<br>Livro e Cia Team</p>
</body>
</html>"#,
            name = to_name,
            url = reset_url
        );

        let message =
            self.build_email(to_email, to_name, "Reset your password - Livro e Cia", body)?;
        self.send(message)?;
        log::info!("Password reset email sent to: {}", to_email);
        Ok(())
    }
}
