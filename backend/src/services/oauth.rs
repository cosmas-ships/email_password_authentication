// auth.rs
use crate::{
    config::Config,
    error::{AppError, Result},
    models::{GoogleTokenResponse, GoogleUserInfo},
};
use reqwest::Client;
use serde_json::json;

#[derive(Clone)]
pub struct OAuthService {
    client: Client,
    config: Config,
}

impl OAuthService {
    pub fn new(config: Config) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// Generate Google OAuth authorization URL
    pub fn get_google_auth_url(&self, state: &str) -> String {
        let scope = "openid email profile";
        
        format!(
            "https://accounts.google.com/o/oauth2/v2/auth?\
             client_id={}&\
             redirect_uri={}&\
             response_type=code&\
             scope={}&\
             state={}&\
             access_type=offline&\
             prompt=consent",
            self.config.google_client_id,
            urlencoding::encode(&self.config.google_redirect_uri),
            urlencoding::encode(scope),
            state
        )
    }

    /// Exchange authorization code for access token
    pub async fn exchange_google_code(&self, code: &str) -> Result<GoogleTokenResponse> {
        let token_url = "https://oauth2.googleapis.com/token";
        
        let params = json!({
            "code": code,
            "client_id": self.config.google_client_id,
            "client_secret": self.config.google_client_secret,
            "redirect_uri": self.config.google_redirect_uri,
            "grant_type": "authorization_code"
        });

        let response = self
            .client
            .post(token_url)
            .json(&params)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to exchange Google code: {:?}", e);
                AppError::InternalServerError("OAuth exchange failed".to_string())
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Google token exchange failed: {}", error_text);
            return Err(AppError::BadRequest("Invalid authorization code".to_string()));
        }

        response
            .json::<GoogleTokenResponse>()
            .await
            .map_err(|e| {
                tracing::error!("Failed to parse Google token response: {:?}", e);
                AppError::InternalServerError("OAuth parsing failed".to_string())
            })
    }

    /// Get Google user info using access token
    pub async fn get_google_user_info(&self, access_token: &str) -> Result<GoogleUserInfo> {
        let user_info_url = "https://www.googleapis.com/oauth2/v2/userinfo";

        let response = self
            .client
            .get(user_info_url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch Google user info: {:?}", e);
                AppError::InternalServerError("Failed to fetch user info".to_string())
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Google user info fetch failed: {}", error_text);
            return Err(AppError::BadRequest("Failed to get user info".to_string()));
        }

        response
            .json::<GoogleUserInfo>()
            .await
            .map_err(|e| {
                tracing::error!("Failed to parse Google user info: {:?}", e);
                AppError::InternalServerError("User info parsing failed".to_string())
            })
    }
}