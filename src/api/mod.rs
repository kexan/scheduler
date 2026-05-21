pub mod auth_middleware;
pub mod auth_token;
pub mod handlers;

use crate::api::auth_middleware::auth_middleware;
use crate::api::auth_token::AdminToken;
use crate::api::handlers::{auth, slots, yougile};
use crate::scheduler::Scheduler;
use crate::yougile::YougileClient;
use axum::{
    Router,
    extract::Path,
    http::StatusCode,
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
};
use include_dir::{Dir, include_dir};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

const STATIC_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/static");

pub fn routes(scheduler: Arc<Scheduler>, yougile: Arc<YougileClient>) -> Router {
    let admin_password = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin123".to_string());
    let admin_token = AdminToken::new();

    let app_state = AppState {
        scheduler: scheduler.clone(),
        yougile: yougile.clone(),
        admin_password: admin_password.clone(),
        admin_token: admin_token.clone(),
    };

    let cors = CorsLayer::permissive();

    let public_routes = Router::new()
        .route("/", get(root_handler))
        .route("/favicon.ico", get(favicon_handler))
        .route("/api/slots", get(slots::get_slots_handler))
        .route("/api/slots/{id}/book", post(slots::book_slot_handler))
        .route("/api/auth/admin", post(auth::admin_auth_handler))
        .route("/api/auth/check", get(auth::check_auth_handler))
        .route("/api/auth/logout", post(auth::logout_handler))
        .route("/{*path}", get(static_handler));

    let protected_routes = Router::new()
        .route("/api/slots", post(slots::create_slot_handler))
        .route("/api/slots/{id}", delete(slots::delete_slot_handler))
        .route("/api/slots/{id}", put(slots::update_slot_handler))
        .route(
            "/api/yougile/settings",
            get(yougile::get_yougile_config_handler),
        )
        .route(
            "/api/yougile/settings",
            put(yougile::update_yougile_config_handler),
        )
        .route("/api/yougile/projects", get(yougile::get_projects_handler))
        .route("/api/yougile/boards", get(yougile::get_boards_handler))
        .route("/api/yougile/columns", get(yougile::get_columns_handler))
        .route("/api/yougile/users", get(yougile::get_users_handler))
        .layer(from_fn_with_state(app_state.clone(), auth_middleware));

    public_routes
        .merge(protected_routes)
        .layer(cors)
        .with_state(app_state)
}

#[derive(Clone)]
pub struct AppState {
    pub scheduler: Arc<Scheduler>,
    pub yougile: Arc<YougileClient>,
    pub admin_password: String,
    pub admin_token: AdminToken,
}

async fn root_handler() -> impl IntoResponse {
    static_handler(Path("index.html".to_string())).await
}

async fn favicon_handler() -> impl IntoResponse {
    static_handler(Path("favicon.ico".to_string())).await
}

async fn static_handler(Path(path): Path<String>) -> Result<Response, (StatusCode, &'static str)> {
    let file = STATIC_DIR.get_file(&path);

    match file {
        Some(file) => {
            let content_type = match std::path::Path::new(&path)
                .extension()
                .and_then(|ext| ext.to_str())
            {
                Some("html") => "text/html",
                Some("css") => "text/css",
                Some("js") => "application/javascript",
                Some("ico") => "image/x-icon",
                Some("png") => "image/png",
                Some("svg") => "image/svg+xml",
                _ => "application/octet-stream",
            };

            let headers = axum::http::HeaderMap::from_iter([(
                axum::http::header::CONTENT_TYPE,
                content_type.parse().expect("content type is a valid MIME type"),
            )]);

            Ok((headers, axum::body::Bytes::from_static(file.contents())).into_response())
        }
        None => Err((StatusCode::NOT_FOUND, "Not Found")),
    }
}
