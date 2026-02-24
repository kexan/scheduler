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

    let cookie = state.admin_token.create_session().await;

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
    let cookie = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    let authenticated = state.admin_token.check_auth(cookie).await.is_ok();
    Json(CheckAuthResponse { authenticated })
}
