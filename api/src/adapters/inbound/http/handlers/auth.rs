use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::{
    adapters::inbound::http::dto::auth::{AuthResponse, LoginRequest, RegisterRequest},
    domain::ports::inbound::auth_service::{
        AuthServicePort, LoginCommand, LogoutCommand, RefreshCommand, RegisterCommand,
    },
    errors::AppError,
};

pub struct AuthState {
    pub service: Arc<dyn AuthServicePort>,
    pub cookie_secure: bool,
    pub cookie_same_site: String,
    pub cookie_max_age_secs: u64,
}

fn auth_cookie(
    token: &str,
    secure: bool,
    same_site: &str,
    max_age_secs: u64,
) -> Result<HeaderValue, AppError> {
    // Validate SameSite against the allowed set to prevent attribute injection.
    let normalized_same_site = if same_site.eq_ignore_ascii_case("strict") {
        "Strict"
    } else if same_site.eq_ignore_ascii_case("lax") {
        "Lax"
    } else if same_site.eq_ignore_ascii_case("none") {
        "None"
    } else {
        return Err(AppError::Internal(anyhow::anyhow!(
            "Invalid COOKIE_SAME_SITE value: {:?}; must be Strict, Lax, or None",
            same_site
        )));
    };

    // SameSite=None requires Secure per spec — enforce it regardless of config.
    let effective_secure = secure || normalized_same_site == "None";
    let secure_flag = if effective_secure { "; Secure" } else { "" };
    HeaderValue::from_str(&format!(
        "token={}; HttpOnly; Path=/; SameSite={}; Max-Age={}{}",
        token, normalized_same_site, max_age_secs, secure_flag
    ))
    .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid Set-Cookie header value: {}", e)))
}

fn clear_cookie() -> HeaderValue {
    // Mirror the SameSite/Secure logic used for the auth cookie so that the
    // clearing Set-Cookie reliably targets the same cookie instance.
    let same_site = std::env::var("COOKIE_SAME_SITE").unwrap_or_else(|_| "Lax".to_string());
    let normalized_same_site = if same_site.eq_ignore_ascii_case("strict") {
        "Strict"
    } else if same_site.eq_ignore_ascii_case("lax") {
        "Lax"
    } else if same_site.eq_ignore_ascii_case("none") {
        "None"
    } else {
        // Fall back to a safe default rather than failing, since this function
        // does not return a Result.
        "Lax"
    };

    // Interpret COOKIE_SECURE in a forgiving way: "true"/"1" enable Secure.
    let secure = std::env::var("COOKIE_SECURE")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false);

    // SameSite=None requires Secure per spec — enforce it regardless of config.
    let effective_secure = secure || normalized_same_site == "None";
    let secure_flag = if effective_secure { "; Secure" } else { "" };

    let value = format!(
        "token=; HttpOnly; Path=/; SameSite={}; Max-Age=0{}",
        normalized_same_site, secure_flag
    );

    HeaderValue::from_str(&value)
        // On any error constructing the header, fall back to the previous behavior.
        .unwrap_or_else(|_| HeaderValue::from_static("token=; HttpOnly; Path=/; Max-Age=0"))
}

/// Extracts the raw JWT from `Authorization: Bearer <token>` or the `token` cookie.
/// The Bearer scheme comparison is case-insensitive per RFC 7235.
fn extract_token(headers: &HeaderMap) -> Option<String> {
    if let Some(auth) = headers.get("authorization") {
        if let Ok(val) = auth.to_str() {
            let mut parts = val.splitn(2, |c: char| c.is_ascii_whitespace());
            if let (Some(scheme), Some(token)) = (parts.next(), parts.next()) {
                if scheme.eq_ignore_ascii_case("bearer") {
                    let token = token.trim();
                    if !token.is_empty() {
                        return Some(token.to_string());
                    }
                }
            }
        }
    }
    if let Some(cookie) = headers.get("cookie") {
        if let Ok(val) = cookie.to_str() {
            for part in val.split(';') {
                if let Some(token) = part.trim().strip_prefix("token=") {
                    let token = token.trim();
                    if !token.is_empty() {
                        return Some(token.to_string());
                    }
                }
            }
        }
    }
    None
}

pub async fn register(
    State(state): State<Arc<AuthState>>,
    Json(body): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    let auth = state
        .service
        .register(RegisterCommand {
            email: body.email,
            password: body.password,
            full_name: body.full_name,
        })
        .await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Set-Cookie",
        auth_cookie(
            &auth.token,
            state.cookie_secure,
            &state.cookie_same_site,
            state.cookie_max_age_secs,
        )?,
    );

    Ok((
        StatusCode::CREATED,
        headers,
        Json(AuthResponse {
            token: auth.token,
            user_id: auth.user_id,
        }),
    ))
}

pub async fn login(
    State(state): State<Arc<AuthState>>,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let auth = state
        .service
        .login(LoginCommand {
            email: body.email,
            password: body.password,
        })
        .await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Set-Cookie",
        auth_cookie(
            &auth.token,
            state.cookie_secure,
            &state.cookie_same_site,
            state.cookie_max_age_secs,
        )?,
    );

    Ok((
        StatusCode::OK,
        headers,
        Json(AuthResponse {
            token: auth.token,
            user_id: auth.user_id,
        }),
    ))
}

pub async fn logout(
    State(state): State<Arc<AuthState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let token = extract_token(&headers).ok_or(AppError::Unauthorized)?;

    state.service.logout(LogoutCommand { token }).await?;

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert("Set-Cookie", clear_cookie());

    Ok((StatusCode::NO_CONTENT, resp_headers))
}

pub async fn refresh(
    State(state): State<Arc<AuthState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let token = extract_token(&headers).ok_or(AppError::Unauthorized)?;

    let auth = state.service.refresh(RefreshCommand { token }).await?;

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(
        "Set-Cookie",
        auth_cookie(
            &auth.token,
            state.cookie_secure,
            &state.cookie_same_site,
            state.cookie_max_age_secs,
        )?,
    );

    Ok((
        StatusCode::OK,
        resp_headers,
        Json(AuthResponse {
            token: auth.token,
            user_id: auth.user_id,
        }),
    ))
}
