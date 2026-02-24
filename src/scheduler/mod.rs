pub mod models;
pub mod storage;

use chrono::NaiveDate;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};
use uuid::Uuid;

use crate::error::{AppError, Result};
pub use crate::scheduler::models::{
    Booking, CreateBooking, CreateTimeSlot, TimeSlot, UpdateTimeSlot,
};
use crate::scheduler::storage::{load_slots, save_slots};

pub struct Scheduler {
    inner: Arc<Mutex<HashMap<Uuid, TimeSlot>>>,
}

impl Scheduler {
    pub async fn new() -> Result<Self> {
        let slots_map: HashMap<Uuid, TimeSlot> = load_slots()
            .await?
            .into_iter()
            .map(|slot| (slot.id, slot))
            .collect();

        let inner = Arc::new(Mutex::new(slots_map));

        Ok(Self { inner })
    }

    async fn do_save(&self) -> Result<()> {
        let slots = self.get_all_slots().await;

        save_slots(&slots)
            .await
            .inspect_err(|e| error!("Failed to save slots: {}", e))
    }

    pub async fn create_slot(&self, create_timeslot: CreateTimeSlot) -> Result<TimeSlot> {
        let slot = {
            let mut inner = self.inner.lock().await;
            let slot = TimeSlot {
                id: Uuid::new_v4(),
                start_time: create_timeslot.start_time,
                end_time: create_timeslot.end_time,
                date: create_timeslot.date,
                is_available: true,
                booking: None,
                yougile_task_id: None,
                completed: false,
            };

            inner.insert(slot.id, slot.clone());
            slot
        };
        self.do_save().await?;

        info!("Created new slot: {} for date {}", slot.id, slot.date);
        Ok(slot)
    }

    pub async fn book_slot(
        &self,
        slot_id: Uuid,
        create_booking: CreateBooking,
        is_admin: bool,
    ) -> Result<TimeSlot> {
        let today = chrono::Local::now().date_naive();

        let slot = {
            let mut inner = self.inner.lock().await;
            match inner.get_mut(&slot_id) {
                Some(slot) if slot.is_available || is_admin => {
                    if slot.date <= today && !is_admin {
                        return Err(AppError::Other(
                            "Нельзя записываться на слоты в текущие и прошедшие даты".to_string(),
                        ));
                    }
                    let booking = Booking {
                        company_name: create_booking.company_name,
                        admin_email: create_booking.admin_email,
                        company_id: create_booking.company_id,
                        download_email: create_booking.download_email,
                        created_at: chrono::Utc::now(),
                    };

                    slot.is_available = false;
                    slot.booking = Some(booking);
                    slot.clone()
                }
                Some(_) => return Err(AppError::SlotAlreadyBooked),
                None => return Err(AppError::SlotNotFound),
            }
        };

        self.do_save().await?;
        info!(
            "Slot {} booked by company: {}",
            slot_id,
            slot.booking
                .as_ref()
                .map(|b| &b.company_name)
                .unwrap_or(&"Unknown".to_string())
        );
        Ok(slot)
    }

    pub async fn get_all_slots(&self) -> Vec<TimeSlot> {
        self.get_slots_in_range(None, None).await
    }

    pub async fn get_slots_in_range(
        &self,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Vec<TimeSlot> {
        let mut slots: Vec<TimeSlot> = {
            let inner = self.inner.lock().await;
            inner
                .values()
                .filter(|slot| {
                    if let Some(from) = from
                        && slot.date < from
                    {
                        return false;
                    }
                    if let Some(to) = to
                        && slot.date > to
                    {
                        return false;
                    }
                    true
                })
                .cloned()
                .collect()
        };

        slots.sort_by(|a, b| a.date.cmp(&b.date).then(a.start_time.cmp(&b.start_time)));
        slots
    }

    pub async fn get_slot(&self, id: Uuid) -> Option<TimeSlot> {
        let inner = self.inner.lock().await;
        inner.get(&id).cloned()
    }

    pub async fn delete_slot(&self, id: Uuid) -> Result<Option<TimeSlot>> {
        let removed = {
            let mut inner = self.inner.lock().await;
            inner.remove(&id)
        };
        if removed.is_some() {
            self.do_save().await?;
            info!("Slot {} deleted successfully", id);
        }
        Ok(removed)
    }

    pub async fn set_yougile_task_id(&self, slot_id: Uuid, task_id: String) -> Result<TimeSlot> {
        let slot = {
            let mut inner = self.inner.lock().await;
            let slot = inner.get_mut(&slot_id).ok_or(AppError::SlotNotFound)?;

            slot.yougile_task_id = Some(task_id);
            slot.clone()
        };
        self.do_save().await?;
        Ok(slot)
    }

    pub async fn update_slot(
        &self,
        id: Uuid,
        update_time_slot: UpdateTimeSlot,
    ) -> Result<TimeSlot> {
        let slot = {
            let mut inner = self.inner.lock().await;
            let slot = inner.get_mut(&id).ok_or(AppError::SlotNotFound)?;

            if let Some(date) = update_time_slot.date {
                slot.date = date;
            }
            if let Some(start_time) = update_time_slot.start_time {
                slot.start_time = start_time;
            }
            if let Some(end_time) = update_time_slot.end_time {
                slot.end_time = end_time;
            }

            if let Some(is_available) = update_time_slot.is_available {
                slot.is_available = is_available;
            }

            if let Some(partial_booking) = update_time_slot.booking {
                let booking = Booking {
                    company_name: partial_booking.company_name,
                    admin_email: partial_booking.admin_email,
                    company_id: partial_booking.company_id,
                    download_email: partial_booking.download_email,
                    created_at: chrono::Utc::now(),
                };
                slot.booking = Some(booking);
                slot.is_available = false;
            } else if update_time_slot.is_available == Some(false) {
                slot.booking = None;
            }

            if let Some(completed) = update_time_slot.completed {
                slot.completed = completed;
            }

            slot.clone()
        };
        self.do_save().await?;

        info!(
            "Slot {} updated - date: {}, available: {}, has_booking: {}",
            slot.id,
            slot.date,
            slot.is_available,
            slot.booking.is_some()
        );

        Ok(slot)
    }
}
