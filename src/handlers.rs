use warp::{Filter, Reply, Rejection};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::models::{Scheduler, CreateSlotRequest, BookingRequest};

pub fn routes(scheduler: Arc<RwLock<Scheduler>>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

    let api = warp::path("api");

    let get_slots = api
        .and(warp::path("slots"))
        .and(warp::path::end())
        .and(warp::get())
        .and(with_scheduler(scheduler.clone()))
        .and_then(get_slots_handler);

    let create_slot = api
        .and(warp::path("slots"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(with_scheduler(scheduler.clone()))
        .and_then(create_slot_handler);

    let book_slot = api
        .and(warp::path("slots"))
        .and(warp::path::param::<Uuid>())
        .and(warp::path("book"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(with_scheduler(scheduler.clone()))
        .and_then(book_slot_handler);

    let delete_slot = api
        .and(warp::path("slots"))
        .and(warp::path::param::<Uuid>())
        .and(warp::path::end())
        .and(warp::delete())
        .and(with_scheduler(scheduler.clone()))
        .and_then(delete_slot_handler);

    let update_slot = api
        .and(warp::path("slots"))
        .and(warp::path::param::<Uuid>())
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .and(with_scheduler(scheduler.clone()))
        .and_then(update_slot_handler);

    let static_files = warp::fs::dir("static");

    get_slots
        .or(create_slot)
        .or(book_slot)
        .or(delete_slot)
        .or(update_slot)
        .or(static_files)
        .with(cors)
}

fn with_scheduler(
    scheduler: Arc<RwLock<Scheduler>>,
) -> impl Filter<Extract = (Arc<RwLock<Scheduler>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || scheduler.clone())
}

async fn get_slots_handler(scheduler: Arc<RwLock<Scheduler>>) -> Result<impl Reply, Rejection> {
    let slots = scheduler.read().await.get_slots();
    Ok(warp::reply::json(&slots))
}

async fn create_slot_handler(
    request: CreateSlotRequest,
    scheduler: Arc<RwLock<Scheduler>>,
) -> Result<impl Reply, Rejection> {
    let slot = scheduler.write().await.create_slot(request);
    Ok(warp::reply::json(&slot))
}

async fn book_slot_handler(
    slot_id: Uuid,
    request: BookingRequest,
    scheduler: Arc<RwLock<Scheduler>>,
) -> Result<Box<dyn Reply>, Rejection> {
    match scheduler.write().await.book_slot(slot_id, request) {
        Ok(slot) => Ok(Box::new(warp::reply::json(&slot))),
        Err(e) => Ok(Box::new(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({"error": e})),
            warp::http::StatusCode::BAD_REQUEST,
        ))),
    }
}

async fn delete_slot_handler(
    slot_id: Uuid,
    scheduler: Arc<RwLock<Scheduler>>,
) -> Result<Box<dyn Reply>, Rejection> {
    let deleted = scheduler.write().await.delete_slot(slot_id);
    if deleted {
        Ok(Box::new(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({"success": true})),
            warp::http::StatusCode::OK,
        )))
    } else {
        Ok(Box::new(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({"error": "Слот не найден"})),
            warp::http::StatusCode::NOT_FOUND,
        )))
    }
}

async fn update_slot_handler(
    slot_id: Uuid,
    request: CreateSlotRequest,
    scheduler: Arc<RwLock<Scheduler>>,
) -> Result<Box<dyn Reply>, Rejection> {
    match scheduler.write().await.update_slot(slot_id, request) {
        Ok(slot) => Ok(Box::new(warp::reply::json(&slot))),
        Err(e) => Ok(Box::new(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({"error": e})),
            warp::http::StatusCode::BAD_REQUEST,
        ))),
    }
}