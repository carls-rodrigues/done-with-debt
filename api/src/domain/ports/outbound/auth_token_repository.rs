use crate::domain::entities::auth_token::AuthToken;
use crate::errors::AppError;
use async_trait::async_trait;

#[async_trait]
pub trait AuthTokenRepository: Send + Sync {
    async fn create(&self, token: AuthToken) -> Result<AuthToken, AppError>;
    async fn find_by_token(&self, token: &str) -> Result<Option<AuthToken>, AppError>;
    async fn revoke_by_token(&self, token: &str) -> Result<(), AppError>;
}
