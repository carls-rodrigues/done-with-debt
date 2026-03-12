use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::user::{Plan, User};
use crate::domain::ports::outbound::user_repository::{FailedLoginOutcome, UserRepository};
use crate::errors::AppError;

pub struct PostgresUserRepository {
    pool: Arc<PgPool>,
}

impl PostgresUserRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    full_name: String,
    email: String,
    password_hash: Option<String>,
    avatar_url: Option<String>,
    email_verified_at: Option<DateTime<Utc>>,
    plan: String,
    failed_attempts: i32,
    locked_until: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<UserRow> for User {
    type Error = AppError;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        let plan = row.plan.parse::<Plan>().map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Invalid plan value in database: {}", e))
        })?;
        Ok(User {
            id: row.id,
            full_name: row.full_name,
            email: row.email,
            password_hash: row.password_hash,
            avatar_url: row.avatar_url,
            email_verified_at: row.email_verified_at,
            plan,
            failed_attempts: row.failed_attempts,
            locked_until: row.locked_until,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, full_name, email, password_hash, avatar_url,
                   email_verified_at, plan, failed_attempts, locked_until,
                   created_at, updated_at
            FROM users
            WHERE email = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(email)
        .fetch_optional(&*self.pool)
        .await?;

        row.map(User::try_from).transpose()
    }

    async fn create(&self, user: User) -> Result<User, AppError> {
        let plan_str = user.plan.as_str();

        sqlx::query(
            r#"
            INSERT INTO users (id, full_name, email, password_hash, avatar_url,
                               email_verified_at, plan, failed_attempts, locked_until,
                               created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(user.id)
        .bind(&user.full_name)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.avatar_url)
        .bind(user.email_verified_at)
        .bind(plan_str)
        .bind(user.failed_attempts)
        .bind(user.locked_until)
        .bind(user.created_at)
        .bind(user.updated_at)
        .execute(&*self.pool)
        .await?;

        Ok(user)
    }

    async fn record_failed_attempt(
        &self,
        user_id: Uuid,
        max_attempts: i32,
        lock_until: DateTime<Utc>,
    ) -> Result<FailedLoginOutcome, AppError> {
        // Atomically increment or lock in a single UPDATE.
        // On lock: failed_attempts is reset to 0 and locked_until is set.
        let row = sqlx::query_as::<_, (i32, Option<DateTime<Utc>>)>(
            r#"
            UPDATE users
            SET
                failed_attempts = CASE
                    WHEN failed_attempts + 1 >= $2 THEN 0
                    ELSE failed_attempts + 1
                END,
                locked_until = CASE
                    WHEN failed_attempts + 1 >= $2 THEN $3
                    ELSE locked_until
                END,
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING failed_attempts, locked_until
            "#,
        )
        .bind(user_id)
        .bind(max_attempts)
        .bind(lock_until)
        .fetch_one(&*self.pool)
        .await?;

        let (new_attempts, new_locked_until) = row;

        // failed_attempts == 0 with locked_until set means the account was just locked
        if let Some(until) = new_locked_until.filter(|_| new_attempts == 0) {
            Ok(FailedLoginOutcome::Locked { until })
        } else {
            Ok(FailedLoginOutcome::Incremented {
                attempts: new_attempts,
            })
        }
    }

    async fn reset_failed_attempts(&self, user_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE users
            SET failed_attempts = 0, locked_until = NULL, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(user_id)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }
}
