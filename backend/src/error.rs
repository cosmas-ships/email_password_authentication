use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    // ===== Existing errors =====
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("User not found")]
    UserNotFound,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("Token revoked")]
    TokenRevoked,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Validation error: {0}")]
    Validation(String),

    // ✅ Updated to include message
    #[error("Internal server error: {0}")]
    InternalServerError(String),

    #[error("Password hashing error")]
    PasswordHashError,

    #[error("JWT error: {0}")]
    JwtError(String),

    // ===== New email verification errors =====
    #[error("Invalid verification code")]
    InvalidVerificationCode,

    #[error("Verification code has expired")]
    VerificationCodeExpired,

    #[error("Verification code already used")]
    VerificationCodeAlreadyUsed,

    #[error("Email not verified")]
    EmailNotVerified,

    #[error("Email already verified")]
    EmailAlreadyVerified,

    #[error("Failed to send email")]
    EmailSendFailed,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            // ===== Existing errors =====
            AppError::Database(ref e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AppError::Redis(ref e) => {
                tracing::error!("Redis error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            AppError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AppError::UserNotFound => (StatusCode::NOT_FOUND, "User not found"),
            AppError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AppError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token expired"),
            AppError::TokenRevoked => (StatusCode::UNAUTHORIZED, "Token revoked"),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AppError::Validation(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            AppError::InternalServerError(ref msg) => {
                tracing::error!("Internal server error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AppError::PasswordHashError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AppError::JwtError(ref e) => {
                tracing::error!("JWT error: {:?}", e);
                (StatusCode::UNAUTHORIZED, "Invalid token")
            }

            // ===== New email verification errors =====
            AppError::InvalidVerificationCode => (StatusCode::BAD_REQUEST, "Invalid verification code"),
            AppError::VerificationCodeExpired => (StatusCode::BAD_REQUEST, "Verification code has expired"),
            AppError::VerificationCodeAlreadyUsed => (StatusCode::BAD_REQUEST, "Verification code already used"),
            AppError::EmailNotVerified => (StatusCode::FORBIDDEN, "Email not verified"),
            AppError::EmailAlreadyVerified => (StatusCode::BAD_REQUEST, "Email already verified"),
            AppError::EmailSendFailed => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to send email"),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

// ===== Conversions for unexpected errors =====
impl From<Box<dyn std::error::Error>> for AppError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        tracing::error!("Unexpected error: {:?}", e);
        AppError::InternalServerError(format!("{:?}", e))
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        tracing::error!("IO error: {:?}", e);
        AppError::InternalServerError(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        tracing::error!("JSON serialization error: {:?}", e);
        AppError::InternalServerError(e.to_string())
    }
}

// ===== Conversions for email errors (lettre) =====
impl From<lettre::error::Error> for AppError {
    fn from(e: lettre::error::Error) -> Self {
        tracing::error!("Email send error: {:?}", e);
        AppError::EmailSendFailed
    }
}

impl From<lettre::address::AddressError> for AppError {
    fn from(e: lettre::address::AddressError) -> Self {
        tracing::error!("Email address error: {:?}", e);
        AppError::EmailSendFailed
    }
}

impl From<lettre::transport::smtp::Error> for AppError {
    fn from(e: lettre::transport::smtp::Error) -> Self {
        tracing::error!("SMTP transport error: {:?}", e);
        AppError::EmailSendFailed
    }
}

// Convenience alias
pub type Result<T> = std::result::Result<T, AppError>;
