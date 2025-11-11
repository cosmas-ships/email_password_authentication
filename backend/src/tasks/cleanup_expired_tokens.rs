use crate::services::token::TokenService;
use std::time::Duration;

pub fn start_token_cleanup_task(token_service: TokenService) {
    tokio::spawn(async move {
        // Run cleanup every hour (3600 seconds)
        let mut interval = tokio::time::interval(Duration::from_secs(3600));
        
        tracing::info!("Token cleanup task started - running every hour");
        
        loop {
            interval.tick().await;
            
            match token_service.cleanup_expired_tokens().await {
                Ok(deleted) => {
                    if deleted > 0 {
                        tracing::info!("Cleaned up {} expired/revoked tokens", deleted);
                    } else {
                        tracing::debug!("Token cleanup completed - no expired tokens found");
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to cleanup expired tokens: {:?}", e);
                }
            }
        }
    });
}