use crate::config::Config;
use crate::error::{AppError, Result};
use lettre::message::{Message, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::{SmtpTransport, Transport};

#[derive(Clone)]
pub struct EmailService {
    mailer: SmtpTransport,
    sender_email: String,
    sender_name: String,
}

impl EmailService {
    pub fn new(config: &Config) -> Result<Self> {
        let creds = Credentials::new(
            config.smtp_username.clone(),
            config.smtp_password.clone(),
        );

        tracing::info!(
            "Initializing SMTP client (host: {}, port: {})",
            config.smtp_host,
            config.smtp_port
        );

        // Detect correct security mode
        let mailer = if config.smtp_port == 465 {
            // Mailtrap Live → implicit SSL
            tracing::info!("Using implicit TLS (SSL) connection on port 465");
            SmtpTransport::relay(&config.smtp_host)
                .map_err(|e| {
                    AppError::InternalServerError(format!("SMTP relay creation failed: {:?}", e))
                })?
                .port(config.smtp_port)
                .credentials(creds)
                .authentication(vec![Mechanism::Plain, Mechanism::Login])
                .build()
        } else {
            // Default → STARTTLS (e.g., Mailtrap send.smtp.mailtrap.io)
            tracing::info!("Using STARTTLS on port {}", config.smtp_port);
            SmtpTransport::relay(&config.smtp_host)
                .map_err(|e| {
                    AppError::InternalServerError(format!("SMTP relay creation failed: {:?}", e))
                })?
                .port(config.smtp_port)
                .credentials(creds)
                .authentication(vec![Mechanism::Plain, Mechanism::Login])
                .build()
        };

        Ok(Self {
            mailer,
            sender_email: config.smtp_from_email.clone(),
            sender_name: config.smtp_from_name.clone(),
        })
    }

    pub async fn send_verification_email(&self, to: &str, code: &str) -> Result<()> {
        let subject = "Verify Your Email Address";
        let body_text = format!(
            "Welcome!\n\nUse this code to verify your email address:\n\n{}\n\nIf you didn't create an account, ignore this message.",
            code
        );
        let body_html = format!(
            "<p>Welcome!</p><p>Your verification code is <b>{}</b>.</p><p>If you didn't create an account, ignore this message.</p>",
            code
        );

        self.send_email(to, subject, &body_text, &body_html).await
    }

    /// Send password reset email with verification code
    pub async fn send_password_reset_email(&self, to: &str, code: &str) -> Result<()> {
        let subject = "Password Reset Request";
        let body_text = format!(
            "Password Reset Request\n\nYou have requested to reset your password.\n\nYour password reset code is: {}\n\nThis code will expire in 15 minutes.\n\nIf you did not request this password reset, please ignore this email and your password will remain unchanged.",
            code
        );
        let body_html = format!(
            "<h2>Password Reset Request</h2><p>You have requested to reset your password.</p><p>Your password reset code is: <strong>{}</strong></p><p>This code will expire in 15 minutes.</p><p>If you did not request this password reset, please ignore this email and your password will remain unchanged.</p>",
            code
        );

        self.send_email(to, subject, &body_text, &body_html).await
    }

    /// Generic email sending method
    async fn send_email(&self, to: &str, subject: &str, body_text: &str, body_html: &str) -> Result<()> {
        let from_address = format!("{} <{}>", self.sender_name, self.sender_email);

        let email = Message::builder()
            .from(from_address.parse().map_err(|e| {
                AppError::InternalServerError(format!("Invalid FROM address: {}", e))
            })?)
            .to(to.parse().map_err(|e| {
                AppError::InternalServerError(format!("Invalid TO address: {}", e))
            })?)
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(SinglePart::plain(body_text.to_string()))
                    .singlepart(SinglePart::html(body_html.to_string())),
            )
            .map_err(|e| AppError::InternalServerError(format!("Failed to build email: {}", e)))?;

        let mailer = self.mailer.clone();
        let to_clone = to.to_string();
        let subject_clone = subject.to_string();

        tokio::task::spawn_blocking(move || mailer.send(&email))
            .await
            .map_err(|e| AppError::InternalServerError(format!("Tokio join error: {}", e)))?
            .map_err(|e| AppError::InternalServerError(format!("SMTP send error: {}", e)))?;

        tracing::info!("Email '{}' sent successfully to {}", subject_clone, to_clone);
        Ok(())
    }
}