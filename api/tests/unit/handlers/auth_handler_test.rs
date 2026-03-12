use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use tower::util::ServiceExt;

use done_with_debt_api::{
    adapters::inbound::http::handlers::auth::{login, logout, refresh, register, AuthState},
    domain::ports::inbound::auth_service::{
        AuthResult, AuthServicePort, LoginCommand, LogoutCommand, RefreshCommand, RegisterCommand,
    },
    errors::AppError,
};

use async_trait::async_trait;
use axum::routing::{delete, post};
use uuid::Uuid;

// ── Stub AuthService ──────────────────────────────────────────────────────────

struct StubAuthService;

#[async_trait]
impl AuthServicePort for StubAuthService {
    async fn register(&self, _cmd: RegisterCommand) -> Result<AuthResult, AppError> {
        Ok(AuthResult {
            token: "test.jwt.token".to_string(),
            user_id: Uuid::nil(),
        })
    }
    async fn login(&self, _cmd: LoginCommand) -> Result<AuthResult, AppError> {
        Ok(AuthResult {
            token: "test.jwt.token".to_string(),
            user_id: Uuid::nil(),
        })
    }
    async fn logout(&self, _cmd: LogoutCommand) -> Result<(), AppError> {
        Ok(())
    }
    async fn refresh(&self, _cmd: RefreshCommand) -> Result<AuthResult, AppError> {
        Ok(AuthResult {
            token: "test.jwt.token".to_string(),
            user_id: Uuid::nil(),
        })
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_router(secure: bool, same_site: &str, expiry_hours: u64) -> Router {
    let state = Arc::new(AuthState {
        service: Arc::new(StubAuthService),
        cookie_secure: secure,
        cookie_same_site: same_site.to_string(),
        cookie_max_age_secs: expiry_hours * 3600,
    });
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/logout", delete(logout))
        .route("/auth/refresh", post(refresh))
        .with_state(state)
}

fn register_body() -> Body {
    Body::from(r#"{"email":"john@example.com","password":"password1","full_name":"John Doe"}"#)
}

fn login_body() -> Body {
    Body::from(r#"{"email":"john@example.com","password":"password1"}"#)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn register_set_cookie_includes_max_age() {
    let app = make_router(false, "Lax", 168);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/register")
                .header("Content-Type", "application/json")
                .body(register_body())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let cookie = response
        .headers()
        .get("set-cookie")
        .expect("Set-Cookie header must be present")
        .to_str()
        .unwrap();
    assert!(
        cookie.contains("Max-Age=604800"),
        "cookie should contain Max-Age=604800 (168h), got: {cookie}"
    );
}

#[tokio::test]
async fn login_set_cookie_includes_max_age() {
    let app = make_router(false, "Lax", 168);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("Content-Type", "application/json")
                .body(login_body())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response
        .headers()
        .get("set-cookie")
        .expect("Set-Cookie header must be present")
        .to_str()
        .unwrap();
    assert!(
        cookie.contains("Max-Age=604800"),
        "cookie should contain Max-Age=604800 (168h), got: {cookie}"
    );
}

#[tokio::test]
async fn cookie_is_not_secure_when_same_site_lax_and_secure_false() {
    let app = make_router(false, "Lax", 168);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("Content-Type", "application/json")
                .body(login_body())
                .unwrap(),
        )
        .await
        .unwrap();

    let cookie = response
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        !cookie.contains("Secure"),
        "cookie must NOT be Secure for SameSite=Lax + secure=false, got: {cookie}"
    );
}

#[tokio::test]
async fn cookie_is_secure_when_cookie_secure_true() {
    let app = make_router(true, "Lax", 168);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("Content-Type", "application/json")
                .body(login_body())
                .unwrap(),
        )
        .await
        .unwrap();

    let cookie = response
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        cookie.contains("Secure"),
        "cookie must be Secure when cookie_secure=true, got: {cookie}"
    );
}

#[tokio::test]
async fn cookie_is_secure_when_same_site_none_even_if_secure_false() {
    // SameSite=None requires Secure per spec — enforce it regardless of config
    let app = make_router(false, "None", 168);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("Content-Type", "application/json")
                .body(login_body())
                .unwrap(),
        )
        .await
        .unwrap();

    let cookie = response
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        cookie.contains("Secure"),
        "cookie must be Secure when SameSite=None, got: {cookie}"
    );
}

#[tokio::test]
async fn invalid_cookie_same_site_returns_500_instead_of_panicking() {
    // A value with a newline makes HeaderValue::from_str fail.
    // The handler must return 500 Internal Server Error, not panic.
    let app = make_router(false, "Lax\r\nInjected: header", 168);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("Content-Type", "application/json")
                .body(login_body())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "invalid cookie config must return 500, not panic"
    );
}

// ── logout coverage ───────────────────────────────────────────────────────────

#[tokio::test]
async fn logout_with_bearer_token_returns_no_content() {
    let app = make_router(false, "Lax", 168);
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/auth/logout")
                .header("Authorization", "Bearer test.jwt.token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn logout_with_cookie_returns_no_content() {
    let app = make_router(false, "Lax", 168);
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/auth/logout")
                .header("Cookie", "token=test.jwt.token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn logout_with_no_auth_returns_unauthorized() {
    let app = make_router(false, "Lax", 168);
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/auth/logout")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ── refresh coverage ──────────────────────────────────────────────────────────

#[tokio::test]
async fn refresh_with_bearer_token_returns_ok() {
    let app = make_router(false, "Lax", 168);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/refresh")
                .header("Authorization", "Bearer test.jwt.token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn refresh_with_cookie_returns_ok() {
    let app = make_router(false, "Lax", 168);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/refresh")
                .header("Cookie", "token=test.jwt.token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn refresh_with_no_auth_returns_unauthorized() {
    let app = make_router(false, "Lax", 168);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/refresh")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ── Empty cookie value ───────────────────────────────────────────────────────

#[tokio::test]
async fn logout_with_empty_cookie_value_returns_unauthorized() {
    // Cookie: token=  (no value) must be treated as "no token present".
    let app = make_router(false, "Lax", 168);
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/auth/logout")
                .header("Cookie", "token=")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "empty token= cookie must be treated as unauthenticated"
    );
}

// ── SameSite validation ───────────────────────────────────────────────────────

#[tokio::test]
async fn cookie_same_site_with_semicolon_injection_returns_500() {
    // A semicolon in COOKIE_SAME_SITE injects extra cookie attributes.
    // The server must reject this instead of emitting a malformed Set-Cookie.
    let app = make_router(false, "Lax; Injected=attribute", 168);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("Content-Type", "application/json")
                .body(login_body())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "semicolon-injected SameSite must return 500, not silently produce a malformed cookie"
    );
}

// ── Bearer scheme is case-insensitive ────────────────────────────────────────

#[tokio::test]
async fn bearer_scheme_is_case_insensitive() {
    // RFC 7235: auth-scheme is case-insensitive. "bearer" must work like "Bearer".
    let app = make_router(false, "Lax", 168);
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/auth/logout")
                .header("Authorization", "bearer test.jwt.token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NO_CONTENT,
        "lowercase 'bearer' scheme must be accepted"
    );
}
