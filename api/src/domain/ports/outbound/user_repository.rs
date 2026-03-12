use crate::domain::entities::user::User;
use crate::errors::AppError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Result of a single atomic failed-login attempt.
pub enum FailedLoginOutcome {
    /// Account is not locked yet; `attempts` is the new running total.
    Incremented { attempts: i32 },
    /// Threshold was reached; account is now locked until this timestamp.
    Locked { until: DateTime<Utc> },
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn create(&self, user: User) -> Result<User, AppError>;

    /// Atomically increments failed_attempts.
    /// If the new count >= `max_attempts`, sets `locked_until` in the same
    /// operation and returns `Locked`. The Postgres adapter must implement
    /// this as a single UPDATE so there is no window between the increment
    /// and the lock write.
    async fn record_failed_attempt(
        &self,
        user_id: Uuid,
        max_attempts: i32,
        lock_until: DateTime<Utc>,
    ) -> Result<FailedLoginOutcome, AppError>;

    /// Resets failed_attempts to 0 and clears locked_until on successful login.
    async fn reset_failed_attempts(&self, user_id: Uuid) -> Result<(), AppError>;
}
