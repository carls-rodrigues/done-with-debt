use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::user::{Plan, User};
use crate::domain::ports::inbound::auth_service::{AuthResult, AuthServicePort, RegisterCommand};
use crate::domain::ports::outbound::user_repository::UserRepository;
use crate::errors::AppError;

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub struct AuthService<U: UserRepository> {
    user_repo: U,
    jwt_secret: String,
    jwt_expiry_hours: i64,
}

impl<U: UserRepository> AuthService<U> {
    pub fn new(user_repo: U, jwt_secret: String, jwt_expiry_hours: i64) -> Self {
        Self {
            user_repo,
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

    fn issue_jwt(&self, user_id: Uuid) -> Result<String, AppError> {
        let exp = (Utc::now() + Duration::hours(self.jwt_expiry_hours)).timestamp() as usize;
        let claims = Claims {
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
}

#[async_trait]
impl<U: UserRepository> AuthServicePort for AuthService<U> {
    async fn register(&self, cmd: RegisterCommand) -> Result<AuthResult, AppError> {
        let email = cmd.email.trim().to_lowercase();
        Self::validate_email(&email)?;
        Self::validate_password(&cmd.password)?;

        if cmd.password != cmd.password_confirmation {
            return Err(AppError::Validation(
                "Passwords do not match".to_string(),
            ));
        }

        if self.user_repo.find_by_email(&email).await?.is_some() {
            return Err(AppError::Conflict("Email already in use".to_string()));
        }

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
            created_at: now,
            updated_at: now,
        };

        let user = self.user_repo.create(user).await?;
        let token = self.issue_jwt(user.id)?;

        Ok(AuthResult {
            token,
            user_id: user.id,
        })
    }
}
