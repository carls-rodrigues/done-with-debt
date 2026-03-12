use std::sync::Arc;

use chrono::Utc;
use testcontainers::clients;
use testcontainers_modules::postgres::Postgres;
use uuid::Uuid;

use done_with_debt_api::{
    adapters::outbound::postgres::user_repository::PostgresUserRepository,
    domain::{
        entities::user::{Plan, User},
        ports::outbound::user_repository::UserRepository,
    },
    errors::AppError,
};

fn make_user(email: &str) -> User {
    let now = Utc::now();
    User {
        id: Uuid::new_v4(),
        email: email.to_string(),
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

#[tokio::test]
async fn create_duplicate_email_returns_conflict() {
    let docker = clients::Cli::default();
    let pg = docker.run(Postgres::default());
    let port = pg.get_host_port_ipv4(5432);

    let url = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", port);
    let pool = sqlx::PgPool::connect(&url).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let repo = PostgresUserRepository::new(Arc::new(pool));

    repo.create(make_user("dupe@example.com")).await.unwrap();
    let result = repo.create(make_user("dupe@example.com")).await;

    assert!(
        matches!(result, Err(AppError::Conflict(_))),
        "expected Conflict, got {:?}",
        result
    );
}
