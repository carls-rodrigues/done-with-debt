use crate::errors::AppError;
use async_trait::async_trait;
use uuid::Uuid;

pub struct RegisterCommand {
    pub email: String,
    pub password: String,
    pub full_name: String,
}

pub struct LoginCommand {
    pub email: String,
    pub password: String,
}

#[derive(Debug)]
pub struct AuthResult {
    pub token: String,
    pub user_id: Uuid,
}

#[async_trait]
pub trait AuthServicePort: Send + Sync {
    async fn register(&self, cmd: RegisterCommand) -> Result<AuthResult, AppError>;
    async fn login(&self, cmd: LoginCommand) -> Result<AuthResult, AppError>;
}
