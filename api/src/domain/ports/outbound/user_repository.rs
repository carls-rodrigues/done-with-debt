use crate::domain::entities::user::User;
use crate::errors::AppError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn create(&self, user: User) -> Result<User, AppError>;
    /// Atomically increments failed_attempts and returns the new count.
    async fn record_failed_attempt(&self, user_id: Uuid) -> Result<i32, AppError>;
    /// Sets locked_until; the Postgres adapter will also reset failed_attempts to 0.
    async fn lock_until(&self, user_id: Uuid, until: DateTime<Utc>) -> Result<(), AppError>;
    /// Resets failed_attempts to 0 and clears locked_until on successful login.
    async fn reset_failed_attempts(&self, user_id: Uuid) -> Result<(), AppError>;
}
