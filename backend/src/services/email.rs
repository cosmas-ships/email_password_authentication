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
            "Welcome!\n\nUse this code to verify your email address:\n\n{}\n\nIf you didn’t create an account, ignore this message.",
            code
        );
        let body_html = format!(
            "<p>Welcome!</p><p>Your verification code is <b>{}</b>.</p><p>If you didn’t create an account, ignore this message.</p>",
            code
        );

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
                    .singlepart(SinglePart::plain(body_text))
                    .singlepart(SinglePart::html(body_html)),
            )
            .map_err(|e| AppError::InternalServerError(format!("Failed to build email: {}", e)))?;

        let mailer = self.mailer.clone();
        let to_clone = to.to_string();

        tokio::task::spawn_blocking(move || mailer.send(&email))
            .await
            .map_err(|e| AppError::InternalServerError(format!("Tokio join error: {}", e)))?
            .map_err(|e| AppError::InternalServerError(format!("SMTP send error: {}", e)))?;

        tracing::info!("Verification email sent successfully to {}", to_clone);
        Ok(())
    }
}
