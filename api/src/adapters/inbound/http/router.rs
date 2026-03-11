use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;

pub fn create_router(_pool: Arc<PgPool>) -> Router {
    Router::new()
}
