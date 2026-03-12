use std::env;

use anyhow::{Context, Result};

/// Converts JWT expiry hours to cookie Max-Age seconds.
/// Returns `None` if the multiplication would overflow `u64`.
pub fn hours_to_max_age_secs(hours: u64) -> Option<u64> {
    hours.checked_mul(3600)
}

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiry_hours: u64,
    pub cookie_secure: bool,
    pub cookie_same_site: String,
    pub allowed_origins: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse::<u16>()
                .context("PORT must be a valid port number (0-65535)")?,
            database_url: env::var("DATABASE_URL").context("DATABASE_URL is required")?,
            jwt_secret: env::var("JWT_SECRET").context("JWT_SECRET is required")?,
            jwt_expiry_hours: env::var("JWT_EXPIRY_HOURS")
                .unwrap_or_else(|_| "168".to_string())
                .parse::<u64>()
                .context("JWT_EXPIRY_HOURS must be a non-negative integer")?,
            cookie_secure: env::var("COOKIE_SECURE")
                .unwrap_or_else(|_| "false".to_string())
                .parse::<bool>()
                .context("COOKIE_SECURE must be 'true' or 'false'")?,
            cookie_same_site: env::var("COOKIE_SAME_SITE").unwrap_or_else(|_| "lax".to_string()),
            allowed_origins: env::var("ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
        })
    }
}
