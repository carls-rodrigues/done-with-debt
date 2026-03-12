use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::auth_token::AuthToken;
use crate::domain::ports::outbound::auth_token_repository::AuthTokenRepository;
use crate::errors::AppError;

pub struct PostgresAuthTokenRepository {
    pool: Arc<PgPool>,
}

impl PostgresAuthTokenRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct AuthTokenRow {
    id: Uuid,
    user_id: Uuid,
    token: String,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl From<AuthTokenRow> for AuthToken {
    fn from(row: AuthTokenRow) -> Self {
        AuthToken {
            id: row.id,
            user_id: row.user_id,
            token: row.token,
            expires_at: row.expires_at,
            created_at: row.created_at,
        }
    }
}

#[async_trait]
impl AuthTokenRepository for PostgresAuthTokenRepository {
    async fn create(&self, token: AuthToken) -> Result<AuthToken, AppError> {
        sqlx::query(
            r#"
            INSERT INTO auth_tokens (id, user_id, token, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(token.id)
        .bind(token.user_id)
        .bind(&token.token)
        .bind(token.expires_at)
        .bind(token.created_at)
        .execute(&*self.pool)
        .await?;

        Ok(token)
    }

    async fn find_by_token(&self, token: &str) -> Result<Option<AuthToken>, AppError> {
        let row = sqlx::query_as::<_, AuthTokenRow>(
            r#"
            SELECT id, user_id, token, expires_at, created_at
            FROM auth_tokens
            WHERE token = $1
            "#,
        )
        .bind(token)
        .fetch_optional(&*self.pool)
        .await?;

        Ok(row.map(AuthToken::from))
    }

    async fn revoke_by_token(&self, token: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM auth_tokens WHERE token = $1")
            .bind(token)
            .execute(&*self.pool)
            .await?;

        Ok(())
    }

    async fn rotate_token(
        &self,
        old_token: &str,
        new_token: AuthToken,
    ) -> Result<AuthToken, AppError> {
        let mut tx = self.pool.begin().await?;

        let deleted = sqlx::query("DELETE FROM auth_tokens WHERE token = $1")
            .bind(old_token)
            .execute(&mut *tx)
            .await?;

        if deleted.rows_affected() == 0 {
            tx.rollback().await?;
            return Err(AppError::Unauthorized);
        }

        sqlx::query(
            r#"
            INSERT INTO auth_tokens (id, user_id, token, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(new_token.id)
        .bind(new_token.user_id)
        .bind(&new_token.token)
        .bind(new_token.expires_at)
        .bind(new_token.created_at)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(new_token)
    }
}
