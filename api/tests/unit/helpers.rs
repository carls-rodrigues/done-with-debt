use async_trait::async_trait;
use done_with_debt_api::domain::{
    entities::auth_token::AuthToken, ports::outbound::auth_token_repository::AuthTokenRepository,
};
use done_with_debt_api::errors::AppError;

/// Drop-in AuthTokenRepository for tests that don't care about token persistence.
/// Silently accepts creates, returns None for lookups, and ignores revocations.
pub struct NoopAuthTokenRepository;

#[async_trait]
impl AuthTokenRepository for NoopAuthTokenRepository {
    async fn create(&self, token: AuthToken) -> Result<AuthToken, AppError> {
        Ok(token)
    }
    async fn find_by_token(&self, _token: &str) -> Result<Option<AuthToken>, AppError> {
        Ok(None)
    }
    async fn revoke_by_token(&self, _token: &str) -> Result<(), AppError> {
        Ok(())
    }
    async fn rotate_token(
        &self,
        _old_token: &str,
        new_token: AuthToken,
    ) -> Result<AuthToken, AppError> {
        Ok(new_token)
    }
}
