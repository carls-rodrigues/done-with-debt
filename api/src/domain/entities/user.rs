use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum Plan {
    Free,
    Premium,
}

impl Plan {
    pub fn as_str(&self) -> &'static str {
        match self {
            Plan::Free => "free",
            Plan::Premium => "premium",
        }
    }
}

impl From<&str> for Plan {
    fn from(s: &str) -> Self {
        match s {
            "premium" => Plan::Premium,
            _ => Plan::Free,
        }
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: Option<String>,
    pub full_name: String,
    pub avatar_url: Option<String>,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub plan: Plan,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
