use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use done_with_debt_api::domain::{
    entities::{
        auth_token::AuthToken,
        user::{Plan, User},
    },
    ports::{
        inbound::auth_service::{AuthServicePort, LoginCommand},
        outbound::{
            auth_token_repository::AuthTokenRepository,
            user_repository::{FailedLoginOutcome, UserRepository},
        },
    },
    services::auth_service::AuthService,
};
use done_with_debt_api::errors::AppError;

// ── In-memory mock repository ─────────────────────────────────────────────────

struct UserRow {
    user: User,
    failed_attempts: i32,
    locked_until: Option<DateTime<Utc>>,
}

struct InMemoryUserRepository {
    rows: Mutex<HashMap<String, UserRow>>, // keyed by email
}

impl InMemoryUserRepository {
    fn with_user(user: User) -> Self {
        let mut map = HashMap::new();
        let email = user.email.clone();
        map.insert(
            email,
            UserRow {
                user,
                failed_attempts: 0,
                locked_until: None,
            },
        );
        Self {
            rows: Mutex::new(map),
        }
    }

    fn failed_attempts_for(&self, email: &str) -> i32 {
        self.rows
            .lock()
            .unwrap()
            .get(email)
            .map(|r| r.failed_attempts)
            .unwrap_or(0)
    }

    fn with_locked_user(user: User, locked_until: DateTime<Utc>) -> Self {
        let mut map = HashMap::new();
        let email = user.email.clone();
        map.insert(
            email,
            UserRow {
                user,
                failed_attempts: 5,
                locked_until: Some(locked_until),
            },
        );
        Self {
            rows: Mutex::new(map),
        }
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let rows = self.rows.lock().unwrap();
        Ok(rows.get(email).map(|row| {
            let mut u = row.user.clone();
            u.failed_attempts = row.failed_attempts;
            u.locked_until = row.locked_until;
            u
        }))
    }

    async fn create(&self, user: User) -> Result<User, AppError> {
        let mut rows = self.rows.lock().unwrap();
        if rows.contains_key(&user.email) {
            return Err(AppError::Conflict("Email already in use".to_string()));
        }
        let email = user.email.clone();
        rows.insert(
            email,
            UserRow {
                user: user.clone(),
                failed_attempts: 0,
                locked_until: None,
            },
        );
        Ok(user)
    }

    async fn record_failed_attempt(
        &self,
        user_id: Uuid,
        max_attempts: i32,
        lock_until: DateTime<Utc>,
    ) -> Result<FailedLoginOutcome, AppError> {
        let mut rows = self.rows.lock().unwrap();
        let row = rows
            .values_mut()
            .find(|r| r.user.id == user_id)
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        row.failed_attempts += 1;
        if row.failed_attempts >= max_attempts {
            row.locked_until = Some(lock_until);
            row.failed_attempts = 0;
            Ok(FailedLoginOutcome::Locked { until: lock_until })
        } else {
            Ok(FailedLoginOutcome::Incremented {
                attempts: row.failed_attempts,
            })
        }
    }

    async fn reset_failed_attempts(&self, user_id: Uuid) -> Result<(), AppError> {
        let mut rows = self.rows.lock().unwrap();
        let row = rows
            .values_mut()
            .find(|r| r.user.id == user_id)
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        row.failed_attempts = 0;
        row.locked_until = None;
        Ok(())
    }
}

// Newtype wrapper around Arc — lets a test retain a reference to inspect state
// after the service has consumed the repo (orphan rules forbid direct
// `impl UserRepository for Arc<InMemoryUserRepository>`).
struct SharedRepo(Arc<InMemoryUserRepository>);

#[async_trait]
impl UserRepository for SharedRepo {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        self.0.find_by_email(email).await
    }
    async fn create(&self, user: User) -> Result<User, AppError> {
        self.0.create(user).await
    }
    async fn record_failed_attempt(
        &self,
        user_id: Uuid,
        max_attempts: i32,
        lock_until: DateTime<Utc>,
    ) -> Result<FailedLoginOutcome, AppError> {
        self.0
            .record_failed_attempt(user_id, max_attempts, lock_until)
            .await
    }
    async fn reset_failed_attempts(&self, user_id: Uuid) -> Result<(), AppError> {
        self.0.reset_failed_attempts(user_id).await
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

const JWT_SECRET: &str = "test_secret_key_32_chars_minimum!";

struct NoopAuthTokenRepository;

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
}

fn make_service(
    repo: InMemoryUserRepository,
) -> AuthService<InMemoryUserRepository, NoopAuthTokenRepository> {
    AuthService::new(
        repo,
        NoopAuthTokenRepository,
        JWT_SECRET.to_string(),
        168_u64,
    )
}

/// Builds a User with an argon2 hash for the provided password.
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

fn login_cmd(email: &str, password: &str) -> LoginCommand {
    LoginCommand {
        email: email.to_string(),
        password: password.to_string(),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn login_with_valid_credentials_returns_token() {
    let user = user_with_password("password1");
    let service = make_service(InMemoryUserRepository::with_user(user));

    let result = service
        .login(login_cmd("john@example.com", "password1"))
        .await;

    assert!(result.is_ok(), "expected Ok, got {:?}", result);
    let auth = result.unwrap();
    assert!(!auth.token.is_empty());
    assert!(auth.user_id != Uuid::nil());
}

#[tokio::test]
async fn login_with_wrong_password_returns_unauthorized() {
    let user = user_with_password("password1");
    let service = make_service(InMemoryUserRepository::with_user(user));

    let result = service
        .login(login_cmd("john@example.com", "wrongpassword"))
        .await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[tokio::test]
async fn login_with_nonexistent_email_returns_same_error_as_wrong_password() {
    let user = user_with_password("password1");
    let service = make_service(InMemoryUserRepository::with_user(user));

    let result = service
        .login(login_cmd("nobody@example.com", "password1"))
        .await;

    // Must be Unauthorized — not NotFound — to prevent email enumeration
    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[tokio::test]
async fn login_with_locked_account_returns_too_many_requests() {
    let user = user_with_password("password1");
    let locked_until = Utc::now() + Duration::minutes(10);
    let service = make_service(InMemoryUserRepository::with_locked_user(user, locked_until));

    let result = service
        .login(login_cmd("john@example.com", "password1"))
        .await;

    assert!(matches!(result, Err(AppError::TooManyRequests(_))));
}

#[tokio::test]
async fn login_wrong_password_increments_failed_attempts() {
    let user = user_with_password("password1");
    let repo = Arc::new(InMemoryUserRepository::with_user(user));
    let service = AuthService::new(
        SharedRepo(Arc::clone(&repo)),
        NoopAuthTokenRepository,
        JWT_SECRET.to_string(),
        168_u64,
    );

    let _ = service.login(login_cmd("john@example.com", "wrong")).await;
    assert_eq!(repo.failed_attempts_for("john@example.com"), 1);

    let _ = service.login(login_cmd("john@example.com", "wrong")).await;
    assert_eq!(repo.failed_attempts_for("john@example.com"), 2);

    let _ = service.login(login_cmd("john@example.com", "wrong")).await;
    assert_eq!(repo.failed_attempts_for("john@example.com"), 3);
}

#[tokio::test]
async fn login_locks_account_after_5_failed_attempts() {
    let user = user_with_password("password1");
    let service = make_service(InMemoryUserRepository::with_user(user));

    // 5 failed attempts
    for _ in 0..5 {
        let _ = service.login(login_cmd("john@example.com", "wrong")).await;
    }

    // 6th attempt (with correct password) must be locked
    let result = service
        .login(login_cmd("john@example.com", "password1"))
        .await;
    assert!(matches!(result, Err(AppError::TooManyRequests(_))));
}

#[tokio::test]
async fn login_resets_failed_attempts_on_success() {
    let user = user_with_password("password1");
    let service = make_service(InMemoryUserRepository::with_user(user));

    // 3 failed attempts then a success
    for _ in 0..3 {
        let _ = service.login(login_cmd("john@example.com", "wrong")).await;
    }
    let _ = service
        .login(login_cmd("john@example.com", "password1"))
        .await;

    // Should be able to fail 4 more times without being locked (counter was reset)
    for _ in 0..4 {
        let _ = service.login(login_cmd("john@example.com", "wrong")).await;
    }
    let result = service.login(login_cmd("john@example.com", "wrong")).await;
    // 5th attempt after reset — should lock now
    assert!(matches!(result, Err(AppError::TooManyRequests(_))));
}
