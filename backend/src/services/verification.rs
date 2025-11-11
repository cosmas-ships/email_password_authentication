use crate::{
    config::Config,
    error::{AppError, Result},
};
use chrono::{Duration, Utc};
use rand::Rng;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum CodeType {
    EmailVerification,
    PasswordReset,
    }

impl CodeType {
    fn as_str(&self) -> &str {
        match self {
            CodeType::EmailVerification => "email_verification",
            CodeType::PasswordReset => "password_reset",
        }
    }
}

#[derive(Clone)]
pub struct VerificationService {
    db: PgPool,
    config: Config,
}

impl VerificationService {
    pub fn new(db: PgPool, config: Config) -> Self {
        Self { db, config }
    }

    /// Generate a 6-digit verification code
    fn generate_code() -> String {
        let mut rng = rand::thread_rng();
        format!("{:06}", rng.gen_range(0..1000000))
    }

    /// Create and store a verification code
    pub async fn create_verification_code(
        &self,
        user_id: Uuid,
        code_type: CodeType,
    ) -> Result<String> {
        let code = Self::generate_code();
        let expires_at = Utc::now() + Duration::seconds(self.config.verification_code_expiry);

        sqlx::query!(
            r#"
            INSERT INTO verification_code (user_id, code, code_type, expires_at)
            VALUES ($1, $2, $3, $4)
            "#,
            user_id,
            code,
            code_type.as_str(),
            expires_at
        )
        .execute(&self.db)
        .await?;

        Ok(code)
    }

    /// Verify a code
    pub async fn verify_code(
        &self,
        user_id: Uuid,
        code: &str,
        code_type: CodeType,
    ) -> Result<()> {
        let result = sqlx::query!(
            r#"
            SELECT id, expires_at, used_at
            FROM verification_code
            WHERE user_id = $1 AND code = $2 AND code_type = $3
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            user_id,
            code,
            code_type.as_str()
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(AppError::InvalidVerificationCode)?;

        // Check if already used
        if result.used_at.is_some() {
            return Err(AppError::VerificationCodeAlreadyUsed);
        }

        // Check if expired
        if result.expires_at < Utc::now() {
            return Err(AppError::VerificationCodeExpired);
        }

        // Mark as used
        sqlx::query!(
            r#"
            UPDATE verification_code
            SET used_at = $1
            WHERE id = $2
            "#,
            Utc::now(),
            result.id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

}