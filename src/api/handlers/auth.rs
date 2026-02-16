use crate::api::auth_token::AdminToken;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use tracing::warn;
use warp::{Rejection, Reply};

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
    request: AdminAuthRequest,
    admin_password: String,
    admin_token: AdminToken,
) -> Result<warp::reply::Response, Rejection> {
    if request.password == admin_password {
        let cookie = admin_token.create_session().await;

        let response = warp::reply::json(&AdminAuthResponse {
            success: true,
            message: "Авторизация успешна".to_string(),
        });

        Ok(warp::reply::with_header(response, "set-cookie", cookie).into_response())
    } else {
        warn!("🚫 Failed admin authentication attempt");
        Err(warp::reject::custom(AppError::Other(
            "Неверный пароль".to_string(),
        )))
    }
}

#[derive(Serialize)]
pub struct CheckAuthResponse {
    pub authenticated: bool,
}

pub async fn check_auth_handler(
    cookie_header: Option<String>,
    admin_token: AdminToken,
) -> Result<warp::reply::Json, Rejection> {
    if admin_token.check_auth(cookie_header).await.is_ok() {
        Ok(warp::reply::json(&CheckAuthResponse {
            authenticated: true,
        }))
    } else {
        Ok(warp::reply::json(&CheckAuthResponse {
            authenticated: false,
        }))
    }
}
