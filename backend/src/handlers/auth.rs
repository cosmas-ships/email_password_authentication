use crate::{
    error::{AppError, Result}, 
    middleware::RequestExt,
    models::{
        ActiveSessionsResponse, AuthResponse, CheckSessionsRequest, LoginRequest, 
        LogoutRequest, LogoutResponse, RefreshRequest, RegisterRequest, UserResponse
    }, 
    services::{password::PasswordService, verification::CodeType}, 
    state::AppState
};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use validator::Validate;

/// Register a new user with email verification
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    // Hash password
    let password_hash = PasswordService::hash_password(&payload.password)?;

    // Create user
    let user = state
        .user_service
        .create_user(&payload.email, &password_hash)
        .await?;

    // Generate verification code
    let code = state
        .verification_service
        .create_verification_code(user.id, CodeType::EmailVerification)
        .await?;

    // Send verification email
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

    // Verify the provided code
    state
        .verification_service
        .verify_code(user.id, &payload.code, CodeType::EmailVerification)
        .await?;

    // Mark email as verified
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

/// Login user (only if verified)
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let user = state.user_service.get_user_by_email(&payload.email).await?;

    if !user.email_verified {
        return Err(AppError::EmailNotVerified);
    }

    let is_valid = PasswordService::verify_password(&payload.password, &user.password_hash)?;
    if !is_valid {
        return Err(AppError::InvalidCredentials);
    }

    let refresh_token_id = Uuid::new_v4();
    let access_token = state.jwt_service.generate_access_token(&user, refresh_token_id)?;
    let refresh_token = state
        .jwt_service
        .generate_refresh_token(user.id, refresh_token_id)?;

    state
        .token_service
        .store_refresh_token(refresh_token_id, user.id, &refresh_token, None, None)
        .await?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".into(),
        expires_in: state.config.access_token_expiry,
    }))
}

/// Refresh access token using a valid refresh token
pub async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>> {
    let refresh_token = payload.refresh_token.clone();

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

    Ok(Json(AuthResponse {
        access_token: new_access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".into(),
        expires_in: state.config.access_token_expiry,
    }))
}

/// Get all active sessions for the current user
pub async fn get_active_sessions(
    State(state): State<AppState>,
    Json(payload): Json<CheckSessionsRequest>,
) -> Result<Json<ActiveSessionsResponse>> {
    let token = &payload.access_token;

    let claims = state.jwt_service.verify_access_token(token)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;
    let current_token_id = Uuid::parse_str(&claims.jti).map_err(|_| AppError::InvalidToken)?;

    let sessions = state
        .token_service
        .get_active_sessions(user_id, current_token_id)
        .await?;

    let total_sessions = sessions.len();

    Ok(Json(ActiveSessionsResponse {
        current_session_id: current_token_id,
        total_sessions,
        sessions,
    }))
}

/// Logout user with option to logout from all devices
pub async fn logout(
    State(state): State<AppState>,
    Json(payload): Json<LogoutRequest>,
) -> Result<Json<LogoutResponse>> {
    let claims = state
        .jwt_service
        .verify_access_token(&payload.access_token)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;

    state
        .token_service
        .blacklist_access_token(&payload.access_token, state.config.access_token_expiry)
        .await?;

    let sessions_revoked = if payload.logout_all {
        state.token_service.revoke_all_user_tokens(user_id).await?
    } else {
        state
            .token_service
            .revoke_token(&payload.refresh_token)
            .await?;
        1
    };

    Ok(Json(LogoutResponse {
        message: if payload.logout_all {
            "Logged out from all devices successfully.".into()
        } else {
            "Logged out from this device.".into()
        },
        sessions_revoked,
    }))
}

/// Get current authenticated user
pub async fn me(State(state): State<AppState>, req: Request) -> Result<Json<UserResponse>> {
    let user_id = req.user_id()?;

    let user = state.user_service.get_user_by_id(user_id).await?;
    Ok(Json(UserResponse::from(user)))
}