use crate::api::AppState;
use crate::error::AppError;
use crate::scheduler::{CreateBooking, CreateTimeSlot, TimeSlot, UpdateTimeSlot};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use chrono::NaiveDate;
use serde::Deserialize;
use tracing::error;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct SlotsQuery {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}

async fn spawn_yougile_sync(state: AppState, slot: TimeSlot) {
    let yougile = state.yougile;
    let scheduler = state.scheduler;

    if slot.yougile_task_id.is_some() {
        if let Err(e) = yougile.update_task(&slot).await {
            error!("Failed to update Yougile task for slot {}: {}", slot.id, e);
        }
    } else {
        match yougile.create_task(&slot).await {
            Ok(task_id) if !task_id.is_empty() => {
                if let Err(e) = scheduler.set_yougile_task_id(slot.id, task_id).await {
                    error!("Failed to update task_id in DB for slot {}: {}", slot.id, e);
                }
            }
            Ok(_) => {}
            Err(e) => error!("Failed to create Yougile task for slot {}: {}", slot.id, e),
        }
    }
}

pub async fn get_slots_handler(
    State(state): State<AppState>,
    Query(query): Query<SlotsQuery>,
) -> Result<Json<Vec<TimeSlot>>, AppError> {
    let slots = state.scheduler.get_slots_in_range(query.from, query.to);
    Ok(Json(slots))
}

pub async fn create_slot_handler(
    State(state): State<AppState>,
    Json(request): Json<CreateTimeSlot>,
) -> Result<Json<TimeSlot>, AppError> {
    let slot = state.scheduler.create_slot(request).await?;
    Ok(Json(slot))
}

pub async fn delete_slot_handler(
    State(state): State<AppState>,
    Path(slot_id): Path<Uuid>,
) -> Result<Json<bool>, AppError> {
    let yougile_task_id = state
        .scheduler
        .get_slot(slot_id)
        .and_then(|s| s.yougile_task_id);

    state.scheduler.delete_slot(slot_id).await?;

    if let Some(task_id) = yougile_task_id {
        tokio::spawn(async move {
            if let Err(e) = state.yougile.delete_task(&task_id).await {
                error!("Failed to delete Yougile task {}: {}", task_id, e);
            }
        });
    }

    Ok(Json(true))
}

pub async fn book_slot_handler(
    State(state): State<AppState>,
    Path(slot_id): Path<Uuid>,
    Json(request): Json<CreateBooking>,
) -> Result<Json<TimeSlot>, AppError> {
    let slot = state.scheduler.book_slot(slot_id, request).await?;

    tokio::spawn(spawn_yougile_sync(state, slot.clone()));

    Ok(Json(slot))
}

pub async fn update_slot_handler(
    State(state): State<AppState>,
    Path(slot_id): Path<Uuid>,
    Json(request): Json<UpdateTimeSlot>,
) -> Result<Json<TimeSlot>, AppError> {
    let slot = state.scheduler.update_slot(slot_id, request).await?;

    tokio::spawn(spawn_yougile_sync(state, slot.clone()));

    Ok(Json(slot))
}
