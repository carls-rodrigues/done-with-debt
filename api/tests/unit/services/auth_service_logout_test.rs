use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use done_with_debt_api::domain::{
    entities::{
        auth_token::AuthToken,
        user::{Plan, User},
    },
    ports::{
        inbound::auth_service::{AuthServicePort, LoginCommand, LogoutCommand},
        outbound::{
            auth_token_repository::AuthTokenRepository,
            user_repository::{FailedLoginOutcome, UserRepository},
        },
    },
    services::auth_service::AuthService,
};
use done_with_debt_api::errors::AppError;

// ── In-memory repositories ────────────────────────────────────────────────────

struct InMemoryUserRepository {
    users: Mutex<HashMap<String, User>>,
}

impl InMemoryUserRepository {
    fn with_user(user: User) -> Self {
        let mut map = HashMap::new();
        map.insert(user.email.clone(), user);
        Self {
            users: Mutex::new(map),
        }
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        Ok(self.users.lock().unwrap().get(email).cloned())
    }
    async fn create(&self, user: User) -> Result<User, AppError> {
        self.users
            .lock()
            .unwrap()
            .insert(user.email.clone(), user.clone());
        Ok(user)
    }
    async fn record_failed_attempt(
        &self,
        _user_id: Uuid,
        _max: i32,
        _lock_until: DateTime<Utc>,
    ) -> Result<FailedLoginOutcome, AppError> {
        Ok(FailedLoginOutcome::Incremented { attempts: 1 })
    }
    async fn reset_failed_attempts(&self, _user_id: Uuid) -> Result<(), AppError> {
        Ok(())
    }
}

struct InMemoryAuthTokenRepository {
    tokens: Mutex<HashMap<String, AuthToken>>, // keyed by token string
}

impl InMemoryAuthTokenRepository {
    fn new() -> Self {
        Self {
            tokens: Mutex::new(HashMap::new()),
        }
    }

    fn token_exists(&self, token: &str) -> bool {
        self.tokens.lock().unwrap().contains_key(token)
    }

    fn token_count(&self) -> usize {
        self.tokens.lock().unwrap().len()
    }
}

#[async_trait]
impl AuthTokenRepository for InMemoryAuthTokenRepository {
    async fn create(&self, token: AuthToken) -> Result<AuthToken, AppError> {
        self.tokens
            .lock()
            .unwrap()
            .insert(token.token.clone(), token.clone());
        Ok(token)
    }

    async fn find_by_token(&self, token: &str) -> Result<Option<AuthToken>, AppError> {
        Ok(self.tokens.lock().unwrap().get(token).cloned())
    }

    async fn revoke_by_token(&self, token: &str) -> Result<(), AppError> {
        let mut tokens = self.tokens.lock().unwrap();
        if tokens.remove(token).is_none() {
            return Err(AppError::NotFound("Token not found".to_string()));
        }
        Ok(())
    }

    async fn rotate_token(
        &self,
        old_token: &str,
        new_token: AuthToken,
    ) -> Result<AuthToken, AppError> {
        let mut tokens = self.tokens.lock().unwrap();
        tokens.remove(old_token);
        tokens.insert(new_token.token.clone(), new_token.clone());
        Ok(new_token)
    }
}

// Newtype wrappers for Arc sharing
struct SharedTokenRepo(Arc<InMemoryAuthTokenRepository>);

#[async_trait]
impl AuthTokenRepository for SharedTokenRepo {
    async fn create(&self, token: AuthToken) -> Result<AuthToken, AppError> {
        self.0.create(token).await
    }
    async fn find_by_token(&self, token: &str) -> Result<Option<AuthToken>, AppError> {
        self.0.find_by_token(token).await
    }
    async fn revoke_by_token(&self, token: &str) -> Result<(), AppError> {
        self.0.revoke_by_token(token).await
    }
    async fn rotate_token(
        &self,
        old_token: &str,
        new_token: AuthToken,
    ) -> Result<AuthToken, AppError> {
        self.0.rotate_token(old_token, new_token).await
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

const JWT_SECRET: &str = "test_secret_key_32_chars_minimum!";

fn make_user(password: &str) -> User {
    use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Argon2, PasswordHasher,
    };
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();
    User {
        id: Uuid::new_v4(),
        email: "john@example.com".to_string(),
        password_hash: Some(hash),
        full_name: "John Doe".to_string(),
        avatar_url: None,
        email_verified_at: None,
        plan: Plan::Free,
        failed_attempts: 0,
        locked_until: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn make_service(
    user: User,
    token_repo: Arc<InMemoryAuthTokenRepository>,
) -> AuthService<InMemoryUserRepository, SharedTokenRepo> {
    AuthService::new(
        InMemoryUserRepository::with_user(user),
        SharedTokenRepo(Arc::clone(&token_repo)),
        JWT_SECRET.to_string(),
        168_u64,
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn login_creates_auth_token_record() {
    let token_repo = Arc::new(InMemoryAuthTokenRepository::new());
    let service = make_service(make_user("password1"), Arc::clone(&token_repo));

    let result = service
        .login(LoginCommand {
            email: "john@example.com".to_string(),
            password: "password1".to_string(),
        })
        .await
        .unwrap();

    assert!(token_repo.token_exists(&result.token));
}

#[tokio::test]
async fn logout_with_valid_token_succeeds_and_revokes_token() {
    let token_repo = Arc::new(InMemoryAuthTokenRepository::new());
    let service = make_service(make_user("password1"), Arc::clone(&token_repo));

    let auth = service
        .login(LoginCommand {
            email: "john@example.com".to_string(),
            password: "password1".to_string(),
        })
        .await
        .unwrap();

    assert!(token_repo.token_exists(&auth.token));

    let result = service
        .logout(LogoutCommand {
            token: auth.token.clone(),
        })
        .await;

    assert!(result.is_ok());
    assert!(!token_repo.token_exists(&auth.token));
}

#[tokio::test]
async fn logout_with_unknown_token_returns_not_found() {
    let token_repo = Arc::new(InMemoryAuthTokenRepository::new());
    let service = make_service(make_user("password1"), Arc::clone(&token_repo));

    let result = service
        .logout(LogoutCommand {
            token: "this.token.doesnt.exist".to_string(),
        })
        .await;

    assert!(matches!(result, Err(AppError::NotFound(_))));
}

#[tokio::test]
async fn multiple_logins_create_separate_tokens() {
    let token_repo = Arc::new(InMemoryAuthTokenRepository::new());
    let service = make_service(make_user("password1"), Arc::clone(&token_repo));

    let auth1 = service
        .login(LoginCommand {
            email: "john@example.com".to_string(),
            password: "password1".to_string(),
        })
        .await
        .unwrap();

    let auth2 = service
        .login(LoginCommand {
            email: "john@example.com".to_string(),
            password: "password1".to_string(),
        })
        .await
        .unwrap();

    assert_ne!(auth1.token, auth2.token);
    assert_eq!(token_repo.token_count(), 2);
}

#[tokio::test]
async fn logout_only_revokes_the_specified_token() {
    let token_repo = Arc::new(InMemoryAuthTokenRepository::new());
    let service = make_service(make_user("password1"), Arc::clone(&token_repo));

    let auth1 = service
        .login(LoginCommand {
            email: "john@example.com".to_string(),
            password: "password1".to_string(),
        })
        .await
        .unwrap();

    let auth2 = service
        .login(LoginCommand {
            email: "john@example.com".to_string(),
            password: "password1".to_string(),
        })
        .await
        .unwrap();

    service
        .logout(LogoutCommand {
            token: auth1.token.clone(),
        })
        .await
        .unwrap();

    // auth1 revoked, auth2 still valid
    assert!(!token_repo.token_exists(&auth1.token));
    assert!(token_repo.token_exists(&auth2.token));
}
