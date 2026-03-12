use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{
    routing::{delete, post},
    Router,
};
use sqlx::PgPool;

use crate::{
    adapters::{
        inbound::http::handlers::auth::{login, logout, refresh, register, AuthState},
        outbound::postgres::{
            auth_token_repository::PostgresAuthTokenRepository,
            user_repository::PostgresUserRepository,
        },
    },
    config::{hours_to_max_age_secs, Config},
    domain::{ports::inbound::auth_service::AuthServicePort, services::auth_service::AuthService},
};

pub fn create_router(pool: Arc<PgPool>, config: &Config) -> Result<Router> {
    let cookie_max_age_secs = hours_to_max_age_secs(config.jwt_expiry_hours)
        .context("JWT_EXPIRY_HOURS * 3600 overflows u64; use a value ≤ 5124095576030431")?;

    let auth_service: Arc<dyn AuthServicePort> = Arc::new(AuthService::new(
        PostgresUserRepository::new(Arc::clone(&pool)),
        PostgresAuthTokenRepository::new(Arc::clone(&pool)),
        config.jwt_secret.clone(),
        config.jwt_expiry_hours,
    ));

    let auth_state = Arc::new(AuthState {
        service: auth_service,
        cookie_secure: config.cookie_secure,
        cookie_same_site: config.cookie_same_site.clone(),
        cookie_max_age_secs,
    });

    let auth_routes = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", delete(logout))
        .route("/refresh", post(refresh))
        .with_state(auth_state);

    Ok(Router::new().nest("/auth", auth_routes))
}
