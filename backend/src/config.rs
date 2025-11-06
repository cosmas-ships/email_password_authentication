use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,

    // JWT configuration
    pub jwt_secret: String,
    pub jwt_issuer: String,
    pub jwt_audience: String,
    pub access_token_expiry: i64,
    pub refresh_token_expiry: i64,

    // Server
    pub host: String,
    pub port: u16,
    pub environment: Environment,
    pub frontend_url: String,

    // SMTP / Email configuration
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_from_email: String,
    pub smtp_from_name: String,

    // Email verification
    pub verification_code_expiry: i64, // in seconds
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        dotenvy::dotenv().ok();

        Ok(Config {
            // Database & Cache
            database_url: env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("Missing DATABASE_URL"))?,
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1/".to_string()),

            // JWT
            jwt_secret: env::var("JWT_SECRET")
                .map_err(|_| anyhow::anyhow!("Missing JWT_SECRET"))?,
            jwt_issuer: env::var("JWT_ISSUER").unwrap_or_else(|_| "auth-backend".to_string()),
            jwt_audience: env::var("JWT_AUDIENCE").unwrap_or_else(|_| "auth-client".to_string()),
            access_token_expiry: env::var("ACCESS_TOKEN_EXPIRY")
                .unwrap_or_else(|_| "900".to_string()) // 15 mins
                .parse()?,
            refresh_token_expiry: env::var("REFRESH_TOKEN_EXPIRY")
                .unwrap_or_else(|_| "604800".to_string()) // 7 days
                .parse()?,

            // Server
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT").unwrap_or_else(|_| "8000".to_string()).parse()?,

            environment: env::var("ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string())
                .parse::<Environment>()
                .unwrap_or(Environment::Development),

            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),

            // SMTP config — Mailtrap-friendly defaults
            smtp_host: env::var("SMTP_HOST")
                .unwrap_or_else(|_| "smtp.mailtrap.io".to_string()),
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".to_string())
                .parse()?,
            smtp_username: env::var("SMTP_USERNAME")
                .map_err(|_| anyhow::anyhow!("Missing SMTP_USERNAME"))?,
            smtp_password: env::var("SMTP_PASSWORD")
                .map_err(|_| anyhow::anyhow!("Missing SMTP_PASSWORD"))?,
            smtp_from_email: env::var("SMTP_FROM_EMAIL")
                .unwrap_or_else(|_| "noreply@neuracreations.com".to_string()),
            smtp_from_name: env::var("SMTP_FROM_NAME")
                .unwrap_or_else(|_| "NeuraCreations Auth".to_string()),

            // Verification
            verification_code_expiry: env::var("VERIFICATION_CODE_EXPIRY")
                .unwrap_or_else(|_| "900".to_string()) // 15 minutes
                .parse()?,
        })
    }

    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }

    pub fn is_development(&self) -> bool {
        self.environment == Environment::Development
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn log_level(&self) -> &str {
        match self.environment {
            Environment::Development => "debug",
            Environment::Production => "info",
        }
    }

    pub fn debug_enabled(&self) -> bool {
        self.is_development()
    }
}

impl Environment {
    pub fn as_str(&self) -> &str {
        match self {
            Environment::Development => "development",
            Environment::Production => "production",
        }
    }
}

impl std::str::FromStr for Environment {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" => Ok(Environment::Development),
            "production" => Ok(Environment::Production),
            _ => Err(anyhow::anyhow!("Invalid environment: {}", s)),
        }
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
