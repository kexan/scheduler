use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
};

use crate::api::AppState;

pub async fn auth_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let cookie = request
        .headers()
        .get("cookie")
        .and_then(|v| v.to_str().ok());

    match state.admin_token.check_auth(cookie) {
        Ok(()) => next.run(request).await,
        Err(_) => (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    }
}
