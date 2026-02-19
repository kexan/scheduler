use crate::api::AppState;
use crate::error::AppError;
use crate::scheduler::{CreateBooking, CreateTimeSlot, TimeSlot, UpdateTimeSlot};
use axum::{
    Json,
    extract::{Path, State},
};
use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSlotRequest {
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBookingRequest {
    pub company_name: String,
    pub admin_email: String,
    pub company_id: String,
    pub download_email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSlotRequest {
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub is_available: Option<bool>,
    pub booking: Option<CreateBookingRequest>,
}

pub async fn get_slots_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<TimeSlot>>, AppError> {
    let slots = state.scheduler.get_slots().await;
    Ok(Json(slots))
}

pub async fn create_slot_handler(
    State(state): State<AppState>,
    Json(request): Json<CreateSlotRequest>,
) -> Result<Json<TimeSlot>, AppError> {
    let create_timeslot = CreateTimeSlot {
        date: request.date,
        start_time: request.start_time,
        end_time: request.end_time,
    };
    let slot = state.scheduler.create_slot(create_timeslot).await?;
    Ok(Json(slot))
}

pub async fn book_slot_handler(
    State(state): State<AppState>,
    Path(slot_id): Path<Uuid>,
    Json(request): Json<CreateBookingRequest>,
) -> Result<Json<TimeSlot>, AppError> {
    let booking_request = CreateBooking {
        company_name: request.company_name,
        admin_email: request.admin_email,
        company_id: request.company_id,
        download_email: request.download_email,
    };

    let slot = state
        .scheduler
        .book_slot(slot_id, booking_request, false)
        .await?;

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
    Json(request): Json<CreateSlotRequest>,
) -> Result<Json<TimeSlot>, AppError> {
    let create_timeslot = CreateTimeSlot {
        date: request.date,
        start_time: request.start_time,
        end_time: request.end_time,
    };
    let slot = state
        .scheduler
        .update_slot(slot_id, create_timeslot)
        .await?;

    if state.yougile.is_enabled().await {
        let yougile = state.yougile.clone();
        let slot_clone = slot.clone();
        tokio::spawn(async move {
            yougile.update_task(&slot_clone).await;
        });
    }

    Ok(Json(slot))
}

pub async fn update_slot_full_handler(
    State(state): State<AppState>,
    Path(slot_id): Path<Uuid>,
    Json(request): Json<UpdateSlotRequest>,
) -> Result<Json<TimeSlot>, AppError> {
    let update_timeslot = UpdateTimeSlot {
        date: Some(request.date),
        start_time: Some(request.start_time),
        end_time: Some(request.end_time),
        is_available: request.is_available,
        booking: request.booking.map(|b| CreateBooking {
            company_name: b.company_name,
            admin_email: b.admin_email,
            company_id: b.company_id,
            download_email: b.download_email,
        }),
    };
    let slot = state
        .scheduler
        .update_slot_full(slot_id, update_timeslot)
        .await?;

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
