use argon2::{
    password_hash::{rand_core::OsRng, PasswordVerifier, SaltString},
    Argon2, PasswordHash, PasswordHasher,
};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::auth_token::AuthToken;
use crate::domain::entities::user::{Plan, User};
use crate::domain::ports::inbound::auth_service::{
    AuthResult, AuthServicePort, LoginCommand, LogoutCommand, RefreshCommand, RegisterCommand,
};
use crate::domain::ports::outbound::auth_token_repository::AuthTokenRepository;
use crate::domain::ports::outbound::user_repository::{FailedLoginOutcome, UserRepository};
use crate::errors::AppError;

const MAX_FAILED_ATTEMPTS: i32 = 5;
const LOCKOUT_MINUTES: i64 = 15;

#[derive(Serialize, Deserialize)]
struct Claims {
    jti: String, // unique token ID — ensures two logins in the same second produce distinct JWTs
    sub: String,
    exp: usize,
}

pub struct AuthService<U: UserRepository, T: AuthTokenRepository> {
    user_repo: U,
    token_repo: T,
    jwt_secret: String,
    jwt_expiry_hours: u64,
}

impl<U: UserRepository, T: AuthTokenRepository> AuthService<U, T> {
    pub fn new(user_repo: U, token_repo: T, jwt_secret: String, jwt_expiry_hours: u64) -> Self {
        Self {
            user_repo,
            token_repo,
            jwt_secret,
            jwt_expiry_hours,
        }
    }

    fn validate_password(password: &str) -> Result<(), AppError> {
        if password.len() < 8 {
            return Err(AppError::Validation(
                "Password must be at least 8 characters".to_string(),
            ));
        }
        if !password.chars().any(|c| c.is_ascii_digit()) {
            return Err(AppError::Validation(
                "Password must contain at least one number".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_email(email: &str) -> Result<(), AppError> {
        use validator::ValidateEmail;
        if !email.validate_email() {
            return Err(AppError::Validation("Invalid email address".to_string()));
        }
        Ok(())
    }

    fn hash_password(password: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Password hashing failed: {}", e)))
    }

    fn verify_password(password: &str, hash: &str) -> bool {
        PasswordHash::new(hash)
            .map(|parsed| {
                Argon2::default()
                    .verify_password(password.as_bytes(), &parsed)
                    .is_ok()
            })
            .unwrap_or(false)
    }

    fn expiry_hours_i64(&self) -> Result<i64, AppError> {
        i64::try_from(self.jwt_expiry_hours)
            .map_err(|_| AppError::Internal(anyhow::anyhow!("JWT expiry hours overflow")))
    }

    fn issue_jwt(&self, user_id: Uuid) -> Result<String, AppError> {
        let expiry_hours = self.expiry_hours_i64()?;
        let exp = (Utc::now() + Duration::hours(expiry_hours))
            .timestamp()
            .try_into()
            .map_err(|_| AppError::Internal(anyhow::anyhow!("JWT exp timestamp overflow")))?;
        let claims = Claims {
            jti: Uuid::new_v4().to_string(),
            sub: user_id.to_string(),
            exp,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(anyhow::anyhow!("JWT error: {}", e)))
    }

    fn decode_jwt(&self, token: &str) -> Result<Uuid, AppError> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized)?;
        Uuid::parse_str(&data.claims.sub).map_err(|_| AppError::Unauthorized)
    }

    async fn persist_token(&self, user_id: Uuid, token: &str) -> Result<(), AppError> {
        let expiry_hours = self.expiry_hours_i64()?;
        self.token_repo
            .create(AuthToken {
                id: Uuid::new_v4(),
                user_id,
                token: token.to_string(),
                expires_at: Utc::now() + Duration::hours(expiry_hours),
                created_at: Utc::now(),
            })
            .await?;
        Ok(())
    }
}

#[async_trait]
impl<U: UserRepository, T: AuthTokenRepository> AuthServicePort for AuthService<U, T> {
    async fn register(&self, cmd: RegisterCommand) -> Result<AuthResult, AppError> {
        let email = cmd.email.trim().to_lowercase();
        Self::validate_email(&email)?;
        Self::validate_password(&cmd.password)?;

        let password_hash = Self::hash_password(&cmd.password)?;
        let now = Utc::now();
        let user = User {
            id: Uuid::new_v4(),
            email,
            password_hash: Some(password_hash),
            full_name: cmd.full_name,
            avatar_url: None,
            email_verified_at: None,
            plan: Plan::Free,
            failed_attempts: 0,
            locked_until: None,
            created_at: now,
            updated_at: now,
        };

        let user = self.user_repo.create(user).await?;
        let token = self.issue_jwt(user.id)?;
        self.persist_token(user.id, &token).await?;

        Ok(AuthResult {
            token,
            user_id: user.id,
        })
    }

    async fn login(&self, cmd: LoginCommand) -> Result<AuthResult, AppError> {
        let email = cmd.email.trim().to_lowercase();

        let user = self
            .user_repo
            .find_by_email(&email)
            .await?
            .ok_or(AppError::Unauthorized)?;

        if let Some(locked_until) = user.locked_until {
            if locked_until > Utc::now() {
                return Err(AppError::TooManyRequests(format!(
                    "Account locked until {}",
                    locked_until.format("%H:%M UTC")
                )));
            }
        }

        let hash = user
            .password_hash
            .as_deref()
            .ok_or(AppError::Unauthorized)?;

        if !Self::verify_password(&cmd.password, hash) {
            let lock_at = Utc::now() + Duration::minutes(LOCKOUT_MINUTES);
            let outcome = self
                .user_repo
                .record_failed_attempt(user.id, MAX_FAILED_ATTEMPTS, lock_at)
                .await?;
            return match outcome {
                FailedLoginOutcome::Locked { until } => Err(AppError::TooManyRequests(format!(
                    "Account locked until {}",
                    until.format("%H:%M UTC")
                ))),
                FailedLoginOutcome::Incremented { .. } => Err(AppError::Unauthorized),
            };
        }

        self.user_repo.reset_failed_attempts(user.id).await?;
        let token = self.issue_jwt(user.id)?;
        self.persist_token(user.id, &token).await?;

        Ok(AuthResult {
            token,
            user_id: user.id,
        })
    }

    async fn logout(&self, cmd: LogoutCommand) -> Result<(), AppError> {
        self.token_repo.revoke_by_token(&cmd.token).await
    }

    async fn refresh(&self, cmd: RefreshCommand) -> Result<AuthResult, AppError> {
        // Validate JWT signature and expiry
        let user_id = self.decode_jwt(&cmd.token)?;

        // Ensure the token is still active (not revoked) and belongs to the same user
        let stored = self
            .token_repo
            .find_by_token(&cmd.token)
            .await?
            .ok_or(AppError::Unauthorized)?;
        if stored.user_id != user_id {
            return Err(AppError::Unauthorized);
        }

        let expiry_hours = self.expiry_hours_i64()?;
        let new_token_str = self.issue_jwt(user_id)?;
        let new_auth_token = AuthToken {
            id: Uuid::new_v4(),
            user_id,
            token: new_token_str.clone(),
            expires_at: Utc::now() + Duration::hours(expiry_hours),
            created_at: Utc::now(),
        };

        self.token_repo
            .rotate_token(&cmd.token, new_auth_token)
            .await?;

        Ok(AuthResult {
            token: new_token_str,
            user_id,
        })
    }
}
