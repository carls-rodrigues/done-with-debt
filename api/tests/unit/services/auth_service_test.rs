use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use uuid::Uuid;

use done_with_debt_api::domain::{
    entities::user::User,
    ports::{
        inbound::auth_service::{AuthServicePort, RegisterCommand},
        outbound::user_repository::UserRepository,
    },
    services::auth_service::AuthService,
};
use done_with_debt_api::errors::AppError;

// ── In-memory mock repository ────────────────────────────────────────────────

struct InMemoryUserRepository {
    users: Mutex<HashMap<String, User>>, // keyed by email
}

impl InMemoryUserRepository {
    fn new() -> Self {
        Self {
            users: Mutex::new(HashMap::new()),
        }
    }

    fn with_user(email: &str, user: User) -> Self {
        let mut map = HashMap::new();
        map.insert(email.to_string(), user);
        Self {
            users: Mutex::new(map),
        }
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let users = self.users.lock().unwrap();
        Ok(users.get(email).cloned())
    }

    async fn create(&self, user: User) -> Result<User, AppError> {
        let mut users = self.users.lock().unwrap();
        if users.contains_key(&user.email) {
            return Err(AppError::Conflict("Email already in use".to_string()));
        }
        users.insert(user.email.clone(), user.clone());
        Ok(user)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_service(repo: InMemoryUserRepository) -> AuthService<InMemoryUserRepository> {
    AuthService::new(
        repo,
        "test_secret_key_32_chars_minimum!".to_string(),
        168_u64,
    )
}

fn valid_command() -> RegisterCommand {
    RegisterCommand {
        email: "john@example.com".to_string(),
        password: "password1".to_string(),
        full_name: "John Doe".to_string(),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn register_with_valid_data_returns_token() {
    let service = make_service(InMemoryUserRepository::new());

    let result = service.register(valid_command()).await;

    assert!(result.is_ok(), "expected Ok, got {:?}", result);
    let auth = result.unwrap();
    assert!(!auth.token.is_empty());
    assert!(auth.user_id != Uuid::nil());
}

#[tokio::test]
async fn register_with_duplicate_email_returns_conflict() {
    let existing = User {
        id: Uuid::new_v4(),
        email: "john@example.com".to_string(),
        password_hash: Some("hash".to_string()),
        full_name: "John Doe".to_string(),
        avatar_url: None,
        email_verified_at: None,
        plan: done_with_debt_api::domain::entities::user::Plan::Free,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let repo = InMemoryUserRepository::with_user("john@example.com", existing);
    let service = make_service(repo);

    let result = service.register(valid_command()).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[tokio::test]
async fn register_with_short_password_returns_validation_error() {
    let service = make_service(InMemoryUserRepository::new());
    let cmd = RegisterCommand {
        password: "pass1".to_string(), // 5 chars — too short
        ..valid_command()
    };

    let result = service.register(cmd).await;

    assert!(matches!(result, Err(AppError::Validation(_))));
}

#[tokio::test]
async fn register_with_password_without_number_returns_validation_error() {
    let service = make_service(InMemoryUserRepository::new());
    let cmd = RegisterCommand {
        password: "passwordonly".to_string(), // no digit
        ..valid_command()
    };

    let result = service.register(cmd).await;

    assert!(matches!(result, Err(AppError::Validation(_))));
}

#[tokio::test]
async fn register_with_invalid_email_returns_validation_error() {
    let service = make_service(InMemoryUserRepository::new());
    let cmd = RegisterCommand {
        email: "not-an-email".to_string(),
        ..valid_command()
    };

    let result = service.register(cmd).await;

    assert!(matches!(result, Err(AppError::Validation(_))));
}
