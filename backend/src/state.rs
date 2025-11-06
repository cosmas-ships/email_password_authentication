use crate::{
    config::Config,
    services::{
        jwt::JwtService,
        token::TokenService,
        users::UserService,         // ✅ fixed: plural `users`
        email::EmailService,
        verification::VerificationService,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub jwt_service: JwtService,
    pub token_service: TokenService,
    pub user_service: UserService,
    pub email_service: EmailService,
    pub verification_service: VerificationService,
}
