use crate::error::AppError;
use crate::scheduler::{BookingRequest, Scheduler};
use crate::yougile::config::YougileSettings;
use crate::yougile::{
    YougileIntegration, create_yougile_task, delete_yougile_task, update_yougile_task,
};
use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;
use uuid::Uuid;
use warp::Rejection;

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

pub async fn get_slots_handler(
    scheduler: Arc<RwLock<Scheduler>>,
) -> Result<warp::reply::Json, Rejection> {
    let slots = scheduler.read().await.get_slots();
    Ok(warp::reply::json(&slots))
}

pub async fn create_slot_handler(
    request: CreateSlotRequest,
    scheduler: Arc<RwLock<Scheduler>>,
) -> Result<warp::reply::Json, Rejection> {
    let mut sched = scheduler.write().await;
    let slot = sched.create_slot(request).await?;

    Ok(warp::reply::json(&slot))
}

pub async fn book_slot_handler(
    slot_id: Uuid,
    request: BookingRequest,
    is_admin: bool,
    scheduler: Arc<RwLock<Scheduler>>,
    yougile_settings: YougileSettings,
) -> Result<warp::reply::Json, Rejection> {
    {
        let sched = scheduler.read().await;
        if let Some(slot) = sched.get_slots().iter().find(|s| s.id == slot_id) {
            if let Err(msg) = validate_slot_for_booking(slot, is_admin) {
                warn!(
                    "📅 Attempt to book past/current slot: {} for date {}",
                    slot_id, slot.date
                );
                return Err(warp::reject::custom(AppError::Other(msg)));
            }
        } else {
            return Err(warp::reject::custom(AppError::SlotNotFound));
        }
    }

    let slot = {
        let mut sched = scheduler.write().await;
        sched.book_slot(slot_id, request.clone()).await
    }?;

    if yougile_settings.enabled {
        let yougile_integration = Arc::new(tokio::sync::RwLock::new(YougileIntegration::new(
            yougile_settings,
        )));
        let slot_clone = slot.clone();
        let scheduler_clone = scheduler.clone();
        tokio::spawn(async move {
            let task_id = create_yougile_task(&yougile_integration, &slot_clone).await;
            if let Some(task_id) = task_id {
                let mut sched = scheduler_clone.write().await;
                let _ = sched.set_yougile_task_id(slot_clone.id, task_id).await;
            }
        });
    }

    Ok(warp::reply::json(&slot))
}

pub async fn delete_slot_handler(
    slot_id: Uuid,
    scheduler: Arc<RwLock<Scheduler>>,
    yougile_settings: YougileSettings,
) -> Result<warp::reply::Json, Rejection> {
    let yougile_task_id = {
        let sched = scheduler.read().await;
        sched
            .get_slot(slot_id)
            .and_then(|s| s.yougile_task_id.clone())
    };

    let deleted = {
        let mut sched = scheduler.write().await;
        sched.delete_slot(slot_id).await?
    };

    if deleted {
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
        warn!("❌ Attempted to delete non-existent slot: {}", slot_id);
        Err(warp::reject::custom(AppError::SlotNotFound))
    }
}

pub async fn update_slot_handler(
    slot_id: Uuid,
    request: CreateSlotRequest,
    scheduler: Arc<RwLock<Scheduler>>,
    yougile_settings: YougileSettings,
) -> Result<warp::reply::Json, Rejection> {
    let mut sched = scheduler.write().await;
    let slot = sched.update_slot(slot_id, request).await?;

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
    scheduler: Arc<RwLock<Scheduler>>,
    yougile_settings: YougileSettings,
) -> Result<warp::reply::Json, Rejection> {
    let mut sched = scheduler.write().await;
    let slot = sched.update_slot_full(slot_id, request).await?;

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
                    let mut sched = scheduler_clone.write().await;
                    let _ = sched.set_yougile_task_id(slot_clone.id, task_id).await;
                }
            } else {
                update_yougile_task(&yougile_integration, &slot_clone).await;
            }
        });
    }

    Ok(warp::reply::json(&slot))
}

fn validate_slot_for_booking(
    slot: &crate::scheduler::TimeSlot,
    is_admin: bool,
) -> Result<(), String> {
    let today = chrono::Local::now().date_naive();
    if !slot.is_available && !is_admin {
        return Err("Слот недоступен для бронирования".to_string());
    }
    if slot.date <= today && !is_admin {
        return Err("Нельзя записываться на слоты в текущие и прошедшие даты".to_string());
    }
    Ok(())
}
