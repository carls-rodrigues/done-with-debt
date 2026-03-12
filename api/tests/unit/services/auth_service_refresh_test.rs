use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Serialize;
use uuid::Uuid;

use done_with_debt_api::domain::{
    entities::{
        auth_token::AuthToken,
        user::{Plan, User},
    },
    ports::{
        inbound::auth_service::{AuthServicePort, LoginCommand, LogoutCommand, RefreshCommand},
        outbound::{
            auth_token_repository::AuthTokenRepository,
            user_repository::{FailedLoginOutcome, UserRepository},
        },
    },
    services::auth_service::AuthService,
};
use done_with_debt_api::errors::AppError;

// ── In-memory token repository ────────────────────────────────────────────────

struct InMemoryAuthTokenRepository {
    tokens: Mutex<HashMap<String, AuthToken>>,
}

impl InMemoryAuthTokenRepository {
    fn new() -> Self {
        Self {
            tokens: Mutex::new(HashMap::new()),
        }
    }

    fn contains(&self, token: &str) -> bool {
        self.tokens.lock().unwrap().contains_key(token)
    }

    fn tamper_user_id(&self, token: &str, new_user_id: Uuid) {
        if let Some(entry) = self.tokens.lock().unwrap().get_mut(token) {
            entry.user_id = new_user_id;
        }
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
        self.tokens.lock().unwrap().remove(token);
        Ok(())
    }

    async fn rotate_token(
        &self,
        old_token: &str,
        new_token: AuthToken,
    ) -> Result<AuthToken, AppError> {
        let mut tokens = self.tokens.lock().unwrap();
        if tokens.remove(old_token).is_none() {
            return Err(AppError::Unauthorized);
        }
        tokens.insert(new_token.token.clone(), new_token.clone());
        Ok(new_token)
    }
}

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

// ── Failing-rotate stub ───────────────────────────────────────────────────────

struct FailingRotateTokenRepo {
    tokens: Mutex<HashMap<String, AuthToken>>,
}

impl FailingRotateTokenRepo {
    fn new() -> Self {
        Self {
            tokens: Mutex::new(HashMap::new()),
        }
    }

    fn contains(&self, token: &str) -> bool {
        self.tokens.lock().unwrap().contains_key(token)
    }
}

#[async_trait]
impl AuthTokenRepository for FailingRotateTokenRepo {
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
        self.tokens.lock().unwrap().remove(token);
        Ok(())
    }

    async fn rotate_token(
        &self,
        _old_token: &str,
        _new_token: AuthToken,
    ) -> Result<AuthToken, AppError> {
        Err(AppError::Internal(anyhow::anyhow!(
            "simulated rotation failure"
        )))
    }
}

struct SharedFailingRepo(Arc<FailingRotateTokenRepo>);

#[async_trait]
impl AuthTokenRepository for SharedFailingRepo {
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

// ── In-memory user repository ─────────────────────────────────────────────────

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
        Ok(user)
    }
    async fn record_failed_attempt(
        &self,
        _user_id: Uuid,
        _max_attempts: i32,
        _lock_until: DateTime<Utc>,
    ) -> Result<FailedLoginOutcome, AppError> {
        Ok(FailedLoginOutcome::Incremented { attempts: 1 })
    }
    async fn reset_failed_attempts(&self, _user_id: Uuid) -> Result<(), AppError> {
        Ok(())
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

const JWT_SECRET: &str = "test_secret_key_32_chars_minimum!";

fn user_with_password(password: &str) -> User {
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

fn make_shared_service(
    user: User,
) -> (
    AuthService<InMemoryUserRepository, SharedTokenRepo>,
    Arc<InMemoryAuthTokenRepository>,
) {
    let token_repo = Arc::new(InMemoryAuthTokenRepository::new());
    let service = AuthService::new(
        InMemoryUserRepository::with_user(user),
        SharedTokenRepo(Arc::clone(&token_repo)),
        JWT_SECRET.to_string(),
        168_u64,
    );
    (service, token_repo)
}

fn expired_jwt(user_id: Uuid) -> String {
    #[derive(Serialize)]
    struct Claims {
        jti: String,
        sub: String,
        exp: usize,
    }
    let claims = Claims {
        jti: Uuid::new_v4().to_string(),
        sub: user_id.to_string(),
        exp: (Utc::now() - Duration::hours(1)).timestamp() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .unwrap()
}

async fn login(service: &AuthService<InMemoryUserRepository, SharedTokenRepo>) -> String {
    service
        .login(LoginCommand {
            email: "john@example.com".to_string(),
            password: "password1".to_string(),
        })
        .await
        .unwrap()
        .token
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn refresh_with_valid_token_returns_new_token() {
    let (service, _) = make_shared_service(user_with_password("password1"));
    let token = login(&service).await;

    let result = service.refresh(RefreshCommand { token }).await;

    assert!(result.is_ok(), "expected Ok, got {:?}", result);
    let auth = result.unwrap();
    assert!(!auth.token.is_empty());
    assert!(auth.user_id != Uuid::nil());
}

#[tokio::test]
async fn refreshed_token_is_different_from_original() {
    let (service, _) = make_shared_service(user_with_password("password1"));
    let original = login(&service).await;

    let result = service
        .refresh(RefreshCommand {
            token: original.clone(),
        })
        .await
        .unwrap();

    assert_ne!(result.token, original);
}

#[tokio::test]
async fn refresh_revokes_old_token() {
    let (service, token_repo) = make_shared_service(user_with_password("password1"));
    let old_token = login(&service).await;

    let _ = service
        .refresh(RefreshCommand {
            token: old_token.clone(),
        })
        .await
        .unwrap();

    assert!(
        !token_repo.contains(&old_token),
        "old token should be revoked after refresh"
    );
}

#[tokio::test]
async fn refresh_persists_new_token() {
    let (service, token_repo) = make_shared_service(user_with_password("password1"));
    let old_token = login(&service).await;

    let new_auth = service
        .refresh(RefreshCommand {
            token: old_token.clone(),
        })
        .await
        .unwrap();

    assert!(
        token_repo.contains(&new_auth.token),
        "new token should be persisted after refresh"
    );
}

#[tokio::test]
async fn refresh_with_revoked_token_returns_unauthorized() {
    let (service, _) = make_shared_service(user_with_password("password1"));
    let token = login(&service).await;

    service
        .logout(LogoutCommand {
            token: token.clone(),
        })
        .await
        .unwrap();

    let result = service.refresh(RefreshCommand { token }).await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[tokio::test]
async fn refresh_with_invalid_jwt_returns_unauthorized() {
    let (service, _) = make_shared_service(user_with_password("password1"));

    let result = service
        .refresh(RefreshCommand {
            token: "not.a.valid.jwt".to_string(),
        })
        .await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[tokio::test]
async fn refresh_with_expired_jwt_returns_unauthorized() {
    let (service, _) = make_shared_service(user_with_password("password1"));
    let expired = expired_jwt(Uuid::new_v4());

    let result = service.refresh(RefreshCommand { token: expired }).await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[tokio::test]
async fn refresh_with_mismatched_user_id_returns_unauthorized() {
    // Token in DB has a different user_id than the one encoded in the JWT sub.
    // The service must detect this inconsistency and return Unauthorized.
    let (service, token_repo) = make_shared_service(user_with_password("password1"));
    let token = login(&service).await;

    token_repo.tamper_user_id(&token, Uuid::new_v4());

    let result = service.refresh(RefreshCommand { token }).await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[tokio::test]
async fn refresh_rotation_failure_leaves_old_token_intact() {
    // If rotate_token fails, the old token must remain active (atomicity guarantee).
    let user = user_with_password("password1");
    let inner = Arc::new(FailingRotateTokenRepo::new());
    let service = AuthService::new(
        InMemoryUserRepository::with_user(user),
        SharedFailingRepo(Arc::clone(&inner)),
        JWT_SECRET.to_string(),
        168_u64,
    );

    let old_token = service
        .login(LoginCommand {
            email: "john@example.com".to_string(),
            password: "password1".to_string(),
        })
        .await
        .unwrap()
        .token;

    assert!(
        inner.contains(&old_token),
        "pre-condition: old token must be seeded"
    );

    let result = service
        .refresh(RefreshCommand {
            token: old_token.clone(),
        })
        .await;

    assert!(
        result.is_err(),
        "refresh must propagate rotate_token failure"
    );
    assert!(
        inner.contains(&old_token),
        "old token must still be present after failed rotation"
    );
}
