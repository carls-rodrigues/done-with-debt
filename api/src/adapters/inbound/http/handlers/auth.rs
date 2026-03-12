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

fn auth_cookie(token: &str, secure: bool, same_site: &str, max_age_secs: u64) -> HeaderValue {
    // SameSite=None requires Secure per spec — enforce it regardless of config.
    let effective_secure = secure || same_site.eq_ignore_ascii_case("none");
    let secure_flag = if effective_secure { "; Secure" } else { "" };
    HeaderValue::from_str(&format!(
        "token={}; HttpOnly; Path=/; SameSite={}; Max-Age={}{}",
        token, same_site, max_age_secs, secure_flag
    ))
    .expect("valid cookie header value")
}

fn clear_cookie() -> HeaderValue {
    HeaderValue::from_static("token=; HttpOnly; Path=/; Max-Age=0")
}

/// Extracts the raw JWT from `Authorization: Bearer <token>` or the `token` cookie.
fn extract_token(headers: &HeaderMap) -> Option<String> {
    if let Some(auth) = headers.get("authorization") {
        if let Ok(val) = auth.to_str() {
            if let Some(token) = val.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
        }
    }
    if let Some(cookie) = headers.get("cookie") {
        if let Ok(val) = cookie.to_str() {
            for part in val.split(';') {
                if let Some(token) = part.trim().strip_prefix("token=") {
                    return Some(token.to_string());
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
        ),
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
        ),
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
        ),
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
