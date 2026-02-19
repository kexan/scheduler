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
    pub success: bool,
    pub message: String,
}

pub async fn admin_auth_handler(
    State(state): State<AppState>,
    Json(request): Json<AdminAuthRequest>,
) -> Result<Response, AppError> {
    if request.password == state.admin_password {
        let cookie = state.admin_token.create_session().await;

        let mut response = Json(AdminAuthResponse {
            success: true,
            message: "Авторизация успешна".to_string(),
        })
        .into_response();

        let headers = response.headers_mut();
        headers.insert(
            axum::http::HeaderName::from_static("set-cookie"),
            cookie.parse().unwrap(),
        );

        Ok(response)
    } else {
        warn!("Failed admin authentication attempt");
        Err(AppError::Other("Неверный пароль".to_string()))
    }
}

#[derive(Serialize)]
pub struct CheckAuthResponse {
    pub authenticated: bool,
}

pub async fn check_auth_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<CheckAuthResponse>, AppError> {
    let cookie = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    if state.admin_token.check_auth(cookie).await.is_ok() {
        Ok(Json(CheckAuthResponse {
            authenticated: true,
        }))
    } else {
        Ok(Json(CheckAuthResponse {
            authenticated: false,
        }))
    }
}
