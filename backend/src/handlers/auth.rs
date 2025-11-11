use crate::{
    error::{AppError, Result},
    middleware::RequestExt,
    models::{
        ActiveSessionsResponse, AuthResponse, LoginRequest, LogoutRequest, LogoutResponse,
        RegisterRequest, UserResponse,
    },
    services::{password::PasswordService, verification::CodeType},
    state::AppState,
};
use axum::{
    Json,
    extract::{Request, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use time::Duration;
use uuid::Uuid;
use validator::Validate;

/// Helper function to create secure HttpOnly cookie
fn create_auth_cookie(
    name: String,
    value: String,
    max_age_seconds: i64,
    secure: bool,
) -> Cookie<'static> {
    Cookie::build((name, value))
        .path("/")
        .max_age(Duration::seconds(max_age_seconds))
        .same_site(SameSite::Strict)
        .http_only(true)
        .secure(secure)
        .build()
}

/// Register a new user with email verification
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let password_hash = PasswordService::hash_password(&payload.password)?;

    let user = state
        .user_service
        .create_user(&payload.email, &password_hash)
        .await?;

    let code = state
        .verification_service
        .create_verification_code(user.id, CodeType::EmailVerification)
        .await?;

    state
        .email_service
        .send_verification_email(&user.email, &code)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            access_token: "".into(),
            refresh_token: "".into(),
            token_type: "Bearer".into(),
            expires_in: 0,
        }),
    ))
}

/// Verify email address
pub async fn verify_email(
    State(state): State<AppState>,
    Json(payload): Json<crate::models::VerifyEmailRequest>,
) -> Result<impl IntoResponse> {
    let user = state.user_service.get_user_by_email(&payload.email).await?;

    if user.email_verified {
        return Err(AppError::EmailAlreadyVerified);
    }

    state
        .verification_service
        .verify_code(user.id, &payload.code, CodeType::EmailVerification)
        .await?;

    state.user_service.mark_email_verified(user.id).await?;

    Ok(Json(crate::models::MessageResponse {
        message: "Email verified successfully. You can now log in.".to_string(),
    }))
}

/// Resend verification code
pub async fn resend_verification_code(
    State(state): State<AppState>,
    Json(payload): Json<crate::models::ResendCodeRequest>,
) -> Result<impl IntoResponse> {
    let user = state.user_service.get_user_by_email(&payload.email).await?;

    if user.email_verified {
        return Err(AppError::EmailAlreadyVerified);
    }

    let code = state
        .verification_service
        .create_verification_code(user.id, CodeType::EmailVerification)
        .await?;

    state
        .email_service
        .send_verification_email(&user.email, &code)
        .await?;

    Ok(Json(crate::models::MessageResponse {
        message: "Verification code sent successfully.".to_string(),
    }))
}

/// Login user (only if verified) - Sets HttpOnly cookies
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user = state.user_service.get_user_by_email(&payload.email).await?;

    if !user.email_verified {
        return Err(AppError::EmailNotVerified);
    }

    let is_valid = PasswordService::verify_password(&payload.password, &user.password_hash)?;
    if !is_valid {
        return Err(AppError::InvalidCredentials);
    }

    let refresh_token_id = Uuid::new_v4();
    let access_token = state
        .jwt_service
        .generate_access_token(&user, refresh_token_id)?;
    let refresh_token = state
        .jwt_service
        .generate_refresh_token(user.id, refresh_token_id)?;

    state
        .token_service
        .store_refresh_token(refresh_token_id, user.id, &refresh_token, None, None)
        .await?;

    let is_secure = state.config.environment.is_production();

    // Create secure HttpOnly cookies
    let access_cookie = create_auth_cookie(
        "accessToken".to_string(),
        access_token.clone(),
        state.config.access_token_expiry,
        is_secure,
    );

    let refresh_cookie = create_auth_cookie(
        "refreshToken".to_string(),
        refresh_token.clone(),
        state.config.refresh_token_expiry,
        is_secure,
    );

    // Build response with cookies
    let mut response = Json(AuthResponse {
        access_token: "set_in_cookie".into(),
        refresh_token: "set_in_cookie".into(),
        token_type: "Bearer".into(),
        expires_in: state.config.access_token_expiry,
    })
    .into_response();

    response.headers_mut().append(
        header::SET_COOKIE,
        access_cookie.to_string().parse().unwrap(),
    );
    response.headers_mut().append(
        header::SET_COOKIE,
        refresh_cookie.to_string().parse().unwrap(),
    );

    Ok(response)
}

/// Refresh access token using a valid refresh token from cookie
pub async fn refresh(State(state): State<AppState>, req: Request) -> Result<impl IntoResponse> {
    let cookies = req
        .headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let refresh_token = cookies
        .split(';')
        .find_map(|cookie| {
            let cookie = cookie.trim();
            cookie.strip_prefix("refreshToken=").map(|v| v.to_string())
        })
        .ok_or(AppError::MissingRefreshToken)?;

    let claims = state.jwt_service.verify_refresh_token(&refresh_token)?;
    let _refresh_record = state
        .token_service
        .verify_refresh_token(&refresh_token)
        .await?;

    let new_token_id = Uuid::new_v4();
    let new_refresh_token = state.jwt_service.generate_refresh_token(
        Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?,
        new_token_id,
    )?;

    state
        .token_service
        .rotate_refresh_token(&refresh_token, new_token_id, &new_refresh_token, None, None)
        .await?;

    let user = state
        .user_service
        .get_user_by_id(Uuid::parse_str(&claims.sub).unwrap())
        .await?;
    let new_access_token = state
        .jwt_service
        .generate_access_token(&user, new_token_id)?;

    let is_secure = state.config.environment.is_production();

    // Create new secure HttpOnly cookies
    let access_cookie = create_auth_cookie(
        "accessToken".to_string(),
        new_access_token.clone(),
        state.config.access_token_expiry,
        is_secure,
    );

    let refresh_cookie = create_auth_cookie(
        "refreshToken".to_string(),
        new_refresh_token.clone(),
        state.config.refresh_token_expiry,
        is_secure,
    );

    let mut response = Json(AuthResponse {
        access_token: "set_in_cookie".into(),
        refresh_token: "set_in_cookie".into(),
        token_type: "Bearer".into(),
        expires_in: state.config.access_token_expiry,
    })
    .into_response();

    response.headers_mut().append(
        header::SET_COOKIE,
        access_cookie.to_string().parse().unwrap(),
    );
    response.headers_mut().append(
        header::SET_COOKIE,
        refresh_cookie.to_string().parse().unwrap(),
    );

    Ok(response)
}

/// Get all active sessions for the current user
pub async fn get_active_sessions(
    State(state): State<AppState>,
    req: Request,
) -> Result<Json<ActiveSessionsResponse>> {
    let cookies = req
        .headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let token = cookies
        .split(';')
        .find_map(|cookie| {
            let cookie = cookie.trim();
            cookie.strip_prefix("accessToken=").map(|v| v.to_string())
        })
        .ok_or(AppError::Unauthorized)?;

    let claims = state.jwt_service.verify_access_token(&token)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;
    let current_token_id = Uuid::parse_str(&claims.jti).map_err(|_| AppError::InvalidToken)?;

    let sessions = state
        .token_service
        .get_active_sessions(user_id, current_token_id)
        .await?;

    Ok(Json(ActiveSessionsResponse {
        current_session_id: current_token_id,
        total_sessions: sessions.len(),
        sessions,
    }))
}

/// Logout user with option to logout from all devices - Clears cookies
pub async fn logout(State(state): State<AppState>, req: Request) -> Result<impl IntoResponse> {
    let cookies = req
        .headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let mut access_token = None;
    let mut refresh_token = None;

    for cookie in cookies.split(';') {
        let cookie = cookie.trim();
        if cookie.starts_with("accessToken=") {
            access_token = Some(cookie.trim_start_matches("accessToken=").to_string());
        } else if cookie.starts_with("refreshToken=") {
            refresh_token = Some(cookie.trim_start_matches("refreshToken=").to_string());
        }
    }

    let access_token = access_token.ok_or(AppError::Unauthorized)?;
    let refresh_token = refresh_token.ok_or(AppError::MissingRefreshToken)?;

    let claims = state.jwt_service.verify_access_token(&access_token)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;

    state
        .token_service
        .blacklist_access_token(&access_token, state.config.access_token_expiry)
        .await?;

    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|_| AppError::BadRequest("Failed to read request body".into()))?;

    let logout_all = if !body_bytes.is_empty() {
        serde_json::from_slice::<LogoutRequest>(&body_bytes)
            .map(|r| r.logout_all)
            .unwrap_or(false)
    } else {
        false
    };

    let sessions_revoked = if logout_all {
        state.token_service.revoke_all_user_tokens(user_id).await?
    } else {
        state.token_service.revoke_token(&refresh_token).await?;
        1
    };

    let clear_access = Cookie::build(("accessToken".to_string(), "".to_string()))
        .path("/")
        .max_age(Duration::seconds(0))
        .http_only(true)
        .build();

    let clear_refresh = Cookie::build(("refreshToken".to_string(), "".to_string()))
        .path("/")
        .max_age(Duration::seconds(0))
        .http_only(true)
        .build();

    let mut response = Json(LogoutResponse {
        message: if logout_all {
            "Logged out from all devices successfully.".into()
        } else {
            "Logged out from this device.".into()
        },
        sessions_revoked,
    })
    .into_response();

    response.headers_mut().append(
        header::SET_COOKIE,
        clear_access.to_string().parse().unwrap(),
    );
    response.headers_mut().append(
        header::SET_COOKIE,
        clear_refresh.to_string().parse().unwrap(),
    );

    Ok(response)
}

/// Get current authenticated user
pub async fn me(State(state): State<AppState>, req: Request) -> Result<Json<UserResponse>> {
    let user_id = req.user_id()?;

    let user = state.user_service.get_user_by_id(user_id).await?;
    Ok(Json(UserResponse::from(user)))
}


// Add these handlers to your auth.rs file

/// Request a password reset code
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(payload): Json<crate::models::ForgotPasswordRequest>,
) -> Result<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Check if user exists
    let user = match state.user_service.get_user_by_email(&payload.email).await {
        Ok(user) => user,
        Err(_) => {
            // Return success even if user doesn't exist (security best practice)
            // This prevents email enumeration attacks
            return Ok(Json(crate::models::MessageResponse {
                message: "If an account exists with this email, a password reset code has been sent.".to_string(),
            }));
        }
    };

    // Only send reset code if email is verified
    if !user.email_verified {
        // Still return success to prevent email enumeration
        return Ok(Json(crate::models::MessageResponse {
            message: "If an account exists with this email, a password reset code has been sent.".to_string(),
        }));
    }

    // Generate password reset code
    let code = state
        .verification_service
        .create_verification_code(user.id, CodeType::PasswordReset)
        .await?;

    // Send password reset email
    state
        .email_service
        .send_password_reset_email(&user.email, &code)
        .await?;

    Ok(Json(crate::models::MessageResponse {
        message: "If an account exists with this email, a password reset code has been sent.".to_string(),
    }))
}

/// Reset password using verification code
pub async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<crate::models::ResetPasswordRequest>,
) -> Result<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Get user by email
    let user = state.user_service.get_user_by_email(&payload.email).await?;

    // Verify the reset code
    state
        .verification_service
        .verify_code(user.id, &payload.code, CodeType::PasswordReset)
        .await?;

    // Hash the new password
    let new_password_hash = PasswordService::hash_password(&payload.new_password)?;

    // Update the password
    state
        .user_service
        .update_password(user.id, &new_password_hash)
        .await?;

    // Revoke all existing sessions for security
    state.token_service.revoke_all_user_tokens(user.id).await?;

    Ok(Json(crate::models::MessageResponse {
        message: "Password reset successfully. Please log in with your new password.".to_string(),
    }))
}