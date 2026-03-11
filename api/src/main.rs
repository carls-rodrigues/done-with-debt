mod config;
mod db;
mod errors;
mod domain;
mod adapters;

use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::from_env().expect("Failed to load config");
    let pool = db::create_pool(&config.database_url).await.expect("Failed to connect to database");

    let app = adapters::inbound::http::router::create_router(Arc::new(pool));

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port))
        .await
        .expect("Failed to bind address");

    tracing::info!("Listening on {}:{}", config.host, config.port);
    axum::serve(listener, app).await.expect("Server error");
}
