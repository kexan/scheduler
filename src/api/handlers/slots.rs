use crate::api::AppState;
use crate::error::AppError;
use crate::scheduler::{CreateBooking, CreateTimeSlot, TimeSlot, UpdateTimeSlot};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use chrono::NaiveDate;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct SlotsQuery {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}

pub async fn get_slots_handler(
    State(state): State<AppState>,
    Query(query): Query<SlotsQuery>,
) -> Result<Json<Vec<TimeSlot>>, AppError> {
    let slots = state
        .scheduler
        .get_slots_in_range(query.from, query.to)
        .await;
    Ok(Json(slots))
}

pub async fn create_slot_handler(
    State(state): State<AppState>,
    Json(request): Json<CreateTimeSlot>,
) -> Result<Json<TimeSlot>, AppError> {
    let slot = state.scheduler.create_slot(request).await?;
    Ok(Json(slot))
}

pub async fn book_slot_handler(
    State(state): State<AppState>,
    Path(slot_id): Path<Uuid>,
    Json(request): Json<CreateBooking>,
) -> Result<Json<TimeSlot>, AppError> {
    let slot = state.scheduler.book_slot(slot_id, request, false).await?;

    if state.yougile.is_enabled().await {
        let yougile = state.yougile.clone();
        let scheduler = state.scheduler.clone();
        let slot_clone = slot.clone();

        tokio::spawn(async move {
            if let Some(task_id) = yougile.create_task(&slot_clone).await {
                scheduler
                    .set_yougile_task_id(slot_clone.id, task_id)
                    .await
                    .ok();
            }
        });
    }

    Ok(Json(slot))
}

pub async fn delete_slot_handler(
    State(state): State<AppState>,
    Path(slot_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let yougile_task_id = state
        .scheduler
        .get_slot(slot_id)
        .await
        .and_then(|s| s.yougile_task_id);
    let deleted = state.scheduler.delete_slot(slot_id).await?;

    if deleted.is_some() {
        if state.yougile.is_enabled().await
            && yougile_task_id.is_some()
            && let Some(task_id) = yougile_task_id
        {
            let yougile = state.yougile.clone();
            tokio::spawn(async move {
                yougile.delete_task(&task_id).await;
            });
        }
        Ok(Json(serde_json::json!({"success": true})))
    } else {
        Err(AppError::SlotNotFound)
    }
}

pub async fn update_slot_handler(
    State(state): State<AppState>,
    Path(slot_id): Path<Uuid>,
    Json(request): Json<UpdateTimeSlot>,
) -> Result<Json<TimeSlot>, AppError> {
    let slot = state.scheduler.update_slot(slot_id, request).await?;

    if state.yougile.is_enabled().await {
        let yougile = state.yougile.clone();
        let scheduler = state.scheduler.clone();
        let slot_clone = slot.clone();
        tokio::spawn(async move {
            if slot_clone.yougile_task_id.is_none() {
                let task_id = yougile.create_task(&slot_clone).await;
                if let Some(task_id) = task_id {
                    scheduler
                        .set_yougile_task_id(slot_clone.id, task_id)
                        .await
                        .ok();
                }
            } else {
                yougile.update_task(&slot_clone).await;
            }
        });
    }

    Ok(Json(slot))
}
