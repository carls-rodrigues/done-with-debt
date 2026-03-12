use std::sync::Arc;

use chrono::{Duration, Utc};
use testcontainers::clients;
use testcontainers_modules::postgres::Postgres;
use uuid::Uuid;

use done_with_debt_api::{
    adapters::outbound::postgres::{
        auth_token_repository::PostgresAuthTokenRepository, user_repository::PostgresUserRepository,
    },
    domain::{
        entities::{
            auth_token::AuthToken,
            user::{Plan, User},
        },
        ports::outbound::{
            auth_token_repository::AuthTokenRepository, user_repository::UserRepository,
        },
    },
    errors::AppError,
};

fn make_user() -> User {
    let now = Utc::now();
    User {
        id: Uuid::new_v4(),
        email: format!("{}@example.com", Uuid::new_v4()),
        password_hash: Some("hash".to_string()),
        full_name: "Test User".to_string(),
        avatar_url: None,
        email_verified_at: None,
        plan: Plan::Free,
        failed_attempts: 0,
        locked_until: None,
        created_at: now,
        updated_at: now,
    }
}

fn make_token(user_id: Uuid) -> AuthToken {
    let now = Utc::now();
    AuthToken {
        id: Uuid::new_v4(),
        user_id,
        token: Uuid::new_v4().to_string(),
        expires_at: now + Duration::hours(168),
        created_at: now,
    }
}

#[tokio::test]
async fn rotate_token_with_already_rotated_old_token_returns_unauthorized() {
    let docker = clients::Cli::default();
    let pg = docker.run(Postgres::default());
    let port = pg.get_host_port_ipv4(5432);

    let url = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", port);
    let pool = Arc::new(sqlx::PgPool::connect(&url).await.unwrap());
    sqlx::migrate!("./migrations").run(&*pool).await.unwrap();

    let user_repo = PostgresUserRepository::new(Arc::clone(&pool));
    let user = user_repo.create(make_user()).await.unwrap();

    let repo = PostgresAuthTokenRepository::new(Arc::clone(&pool));

    let old = make_token(user.id);
    repo.create(old.clone()).await.unwrap();

    // First rotation succeeds: old is deleted, new1 is inserted.
    let new1 = make_token(user.id);
    repo.rotate_token(&old.token, new1).await.unwrap();

    // Second rotation with the same old token must fail: old is already gone.
    let new2 = make_token(user.id);
    let result = repo.rotate_token(&old.token, new2).await;

    assert!(
        matches!(result, Err(AppError::Unauthorized)),
        "rotating an already-rotated token must return Unauthorized, got {:?}",
        result
    );
}
