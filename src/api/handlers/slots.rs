use crate::error::AppError;
use crate::scheduler::{BookingRequest, Scheduler};
use crate::yougile::config::YougileSettings;
use crate::yougile::{
    YougileIntegration, create_yougile_task, delete_yougile_task, update_yougile_task,
};
use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use warp::Rejection;
use warp::reply::Json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSlotRequest {
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSlotRequest {
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub is_available: Option<bool>,
    pub booking: Option<BookingRequest>,
}

pub async fn get_slots_handler(scheduler: Arc<Scheduler>) -> Result<Json, Rejection> {
    let slots = scheduler.get_slots().await;
    Ok(warp::reply::json(&slots))
}

pub async fn create_slot_handler(
    request: CreateSlotRequest,
    scheduler: Arc<Scheduler>,
) -> Result<Json, Rejection> {
    let slot = scheduler.create_slot(request).await?;
    Ok(warp::reply::json(&slot))
}

pub async fn book_slot_handler(
    slot_id: Uuid,
    request: BookingRequest,
    is_admin: bool,
    scheduler: Arc<Scheduler>,
    yougile_settings: YougileSettings,
) -> Result<Json, Rejection> {
    let slot = scheduler.book_slot(slot_id, request, is_admin).await?;

    if yougile_settings.enabled {
        let yougile_integration = Arc::new(tokio::sync::RwLock::new(YougileIntegration::new(
            yougile_settings,
        )));
        let slot_clone = slot.clone();
        let scheduler_clone = scheduler.clone();
        tokio::spawn(async move {
            let task_id = create_yougile_task(&yougile_integration, &slot_clone).await;
            if let Some(task_id) = task_id {
                let _ = scheduler_clone
                    .set_yougile_task_id(slot_clone.id, task_id)
                    .await;
            }
        });
    }

    Ok(warp::reply::json(&slot))
}

pub async fn delete_slot_handler(
    slot_id: Uuid,
    scheduler: Arc<Scheduler>,
    yougile_settings: YougileSettings,
) -> Result<Json, Rejection> {
    let yougile_task_id = scheduler
        .get_slot(slot_id)
        .await
        .and_then(|s| s.yougile_task_id);
    let deleted = scheduler.delete_slot(slot_id).await?;

    if deleted.is_some() {
        if yougile_settings.enabled
            && let Some(task_id) = yougile_task_id
        {
            let yougile_integration = Arc::new(tokio::sync::RwLock::new(YougileIntegration::new(
                yougile_settings,
            )));
            tokio::spawn(async move {
                delete_yougile_task(&yougile_integration, &task_id).await;
            });
        }
        Ok(warp::reply::json(&serde_json::json!({"success": true})))
    } else {
        Err(warp::reject::custom(AppError::SlotNotFound))
    }
}

pub async fn update_slot_handler(
    slot_id: Uuid,
    request: CreateSlotRequest,
    scheduler: Arc<Scheduler>,
    yougile_settings: YougileSettings,
) -> Result<Json, Rejection> {
    let slot = scheduler.update_slot(slot_id, request).await?;

    if yougile_settings.enabled {
        let yougile_integration = Arc::new(tokio::sync::RwLock::new(YougileIntegration::new(
            yougile_settings,
        )));
        let slot_clone = slot.clone();
        tokio::spawn(async move {
            update_yougile_task(&yougile_integration, &slot_clone).await;
        });
    }

    Ok(warp::reply::json(&slot))
}

pub async fn update_slot_full_handler(
    slot_id: Uuid,
    request: UpdateSlotRequest,
    scheduler: Arc<Scheduler>,
    yougile_settings: YougileSettings,
) -> Result<Json, Rejection> {
    let slot = scheduler.update_slot_full(slot_id, request).await?;

    if yougile_settings.enabled {
        let yougile_integration = Arc::new(tokio::sync::RwLock::new(YougileIntegration::new(
            yougile_settings,
        )));
        let slot_clone = slot.clone();
        let scheduler_clone = scheduler.clone();
        tokio::spawn(async move {
            if slot_clone.yougile_task_id.is_none() {
                let task_id = create_yougile_task(&yougile_integration, &slot_clone).await;
                if let Some(task_id) = task_id {
                    let _ = scheduler_clone
                        .set_yougile_task_id(slot_clone.id, task_id)
                        .await;
                }
            } else {
                update_yougile_task(&yougile_integration, &slot_clone).await;
            }
        });
    }

    Ok(warp::reply::json(&slot))
}
