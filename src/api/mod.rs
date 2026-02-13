pub mod auth_token;
pub mod handlers;

use crate::api::auth_token::AdminToken;
use crate::api::handlers::auth::{admin_auth_handler, check_auth_handler};
use crate::api::handlers::slots::{
    CreateSlotRequest, UpdateSlotRequest, book_slot_handler, create_slot_handler,
    delete_slot_handler, get_slots_handler, update_slot_full_handler, update_slot_handler,
};
use crate::api::handlers::yougile::{
    get_yougile_settings_handler, test_yougile_connection_handler, update_yougile_settings_handler,
};

use crate::error::handle_rejection;
use crate::scheduler::Scheduler;
use crate::web;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use warp::{Filter, Reply};

pub fn routes(
    scheduler: Arc<RwLock<Scheduler>>,
) -> impl Filter<Extract = impl Reply, Error = std::convert::Infallible> + Clone {
    let admin_password = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin123".to_string());
    let admin_token = AdminToken::new();
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allow_credentials(true);

    let admin_auth_required = warp::header::optional::<String>("cookie")
        .and(with_admin_token(admin_token.clone()))
        .and_then(|cookie: Option<String>, token: AdminToken| async move {
            token.check_auth(cookie).await.map_err(warp::reject::custom)
        })
        .untuple_one();

    let admin_auth_check = warp::header::optional::<String>("cookie")
        .and(with_admin_token(admin_token.clone()))
        .and_then(|cookie: Option<String>, token: AdminToken| async move {
            Ok::<_, warp::Rejection>(token.check_auth(cookie).await.is_ok())
        });

    let yougile_settings_filter = warp::any().and_then(load_yougile_settings_filter);

    let api = warp::path("api");

    let get_slots = api
        .and(warp::path("slots"))
        .and(warp::path::end())
        .and(warp::get())
        .and(with_scheduler(scheduler.clone()))
        .and_then(get_slots_handler);

    let admin_auth = api
        .and(warp::path("auth"))
        .and(warp::path("admin"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(with_admin_password(admin_password.clone()))
        .and(with_admin_token(admin_token.clone()))
        .and_then(admin_auth_handler);

    let check_auth = api
        .and(warp::path("auth"))
        .and(warp::path("check"))
        .and(warp::path::end())
        .and(warp::get())
        .and(warp::header::optional::<String>("cookie"))
        .and(with_admin_token(admin_token.clone()))
        .and_then(check_auth_handler);

    let create_slot = api
        .and(warp::path("slots"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(admin_auth_required.clone())
        .and(with_scheduler(scheduler.clone()))
        .and_then(create_slot_handler);

    let book_slot = api
        .and(warp::path("slots"))
        .and(warp::path::param::<Uuid>())
        .and(warp::path("book"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(admin_auth_check)
        .and(with_scheduler(scheduler.clone()))
        .and(yougile_settings_filter)
        .and_then(book_slot_handler);

    let delete_slot = api
        .and(warp::path("slots"))
        .and(warp::path::param::<Uuid>())
        .and(warp::path::end())
        .and(warp::delete())
        .and(admin_auth_required.clone())
        .and(with_scheduler(scheduler.clone()))
        .and(yougile_settings_filter)
        .and_then(delete_slot_handler);

    let update_slot = api
        .and(warp::path("slots"))
        .and(warp::path::param::<Uuid>())
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json::<CreateSlotRequest>())
        .and(admin_auth_required.clone())
        .and(with_scheduler(scheduler.clone()))
        .and(yougile_settings_filter)
        .and_then(update_slot_handler);

    let update_slot_full = api
        .and(warp::path("slots"))
        .and(warp::path::param::<Uuid>())
        .and(warp::path("full"))
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json::<UpdateSlotRequest>())
        .and(admin_auth_required.clone())
        .and(with_scheduler(scheduler.clone()))
        .and(yougile_settings_filter)
        .and_then(update_slot_full_handler);

    let get_yougile_settings = api
        .and(warp::path("yougile"))
        .and(warp::path("settings"))
        .and(warp::path::end())
        .and(warp::get())
        .and(admin_auth_required.clone())
        .and(yougile_settings_filter)
        .and_then(get_yougile_settings_handler);

    let update_yougile_settings = api
        .and(warp::path("yougile"))
        .and(warp::path("settings"))
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json::<handlers::YougileSettings>())
        .and(admin_auth_required.clone())
        .and_then(update_yougile_settings_handler);

    let test_yougile_connection = api
        .and(warp::path("yougile"))
        .and(warp::path("test"))
        .and(warp::path::end())
        .and(warp::post())
        .and(admin_auth_required.clone())
        .and(yougile_settings_filter)
        .and_then(test_yougile_connection_handler);

    let static_route = warp::path::tail().and_then(web::static_handler);

    get_slots
        .or(admin_auth)
        .or(check_auth)
        .or(create_slot)
        .or(book_slot)
        .or(delete_slot)
        .or(update_slot)
        .or(update_slot_full)
        .or(get_yougile_settings)
        .or(update_yougile_settings)
        .or(test_yougile_connection)
        .or(static_route)
        .with(cors)
        .recover(handle_rejection)
}

fn with_scheduler(
    scheduler: Arc<RwLock<Scheduler>>,
) -> impl Filter<Extract = (Arc<RwLock<Scheduler>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || scheduler.clone())
}

fn with_admin_password(
    password: String,
) -> impl Filter<Extract = (String,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || password.clone())
}

fn with_admin_token(
    token: AdminToken,
) -> impl Filter<Extract = (AdminToken,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || token.clone())
}

async fn load_yougile_settings_filter()
-> std::result::Result<handlers::YougileSettings, warp::Rejection> {
    crate::yougile::config::load_yougile_settings()
        .await
        .map_err(warp::reject::custom)
}
