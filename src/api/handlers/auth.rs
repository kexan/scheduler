use crate::api::AppState;
use crate::error::AppError;
use axum::{
    Json,
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Deserialize)]
pub struct AdminAuthRequest {
    pub password: String,
}

#[derive(Serialize)]
pub struct AdminAuthResponse {
    pub message: String,
}

#[derive(Serialize)]
pub struct CheckAuthResponse {
    pub authenticated: bool,
}

pub async fn admin_auth_handler(
    State(state): State<AppState>,
    Json(request): Json<AdminAuthRequest>,
) -> Result<Response, AppError> {
    if request.password != state.admin_password {
        warn!("Failed admin authentication attempt");
        return Err(AppError::Unauthorized);
    }

    let cookie = state.admin_token.create_session();

    let mut response = Json(AdminAuthResponse {
        message: "Авторизация успешна".to_string(),
    })
    .into_response();

    response.headers_mut().insert(
        axum::http::HeaderName::from_static("set-cookie"),
        cookie
            .parse()
            .expect("session cookie is always a valid header value"),
    );

    Ok(response)
}

pub async fn check_auth_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<CheckAuthResponse> {
    let cookie = headers.get("cookie").and_then(|v| v.to_str().ok());
    let authenticated = state.admin_token.check_auth(cookie).is_ok();
    Json(CheckAuthResponse { authenticated })
}

pub async fn logout_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let cookie = headers.get("cookie").and_then(|v| v.to_str().ok());

    let token = cookie.and_then(|c| c.split(';').find_map(|cookie| {
        let (key, value) = cookie.trim().split_once('=')?;
        if key == "admin_token" {
            Some(value)
        } else {
            None
        }
    }));

    if let Some(token) = token {
        state.admin_token.logout(token);
    }

    let mut response = Json(serde_json::json!({
        "message": "Выход выполнен"
    }))
    .into_response();

    response.headers_mut().insert(
        axum::http::HeaderName::from_static("set-cookie"),
        "admin_token=; HttpOnly; Secure; SameSite=Strict; Max-Age=0; Path=/"
            .parse()
            .expect("clear cookie is always a valid header value"),
    );

    Ok(response)
}
