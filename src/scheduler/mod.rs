pub mod models;
pub mod storage;

use arc_swap::ArcSwap;
use chrono::{NaiveDate, NaiveTime};
use std::sync::Arc;
use std::{
    collections::{BTreeMap, HashMap},
    ops::Bound,
};
use tokio::sync::Mutex;
use tracing::info;
use uuid::Uuid;

use crate::error::{AppError, Result};
pub use crate::scheduler::models::{
    Booking, CreateBooking, CreateTimeSlot, TimeSlot, UpdateTimeSlot,
};
use crate::scheduler::storage::{load_slots, save_slots};

const DEFAULT_SLOTS_PATH: &str = "data/slots.json";

pub struct Scheduler {
    state: ArcSwap<SchedulerState>,
    write_lock: Mutex<()>,
    path: String,
}

#[derive(Clone)]
struct SchedulerState {
    slots: BTreeMap<SlotKey, Arc<TimeSlot>>,
    by_id: HashMap<Uuid, SlotKey>,
}

type SlotKey = (NaiveDate, NaiveTime, Uuid);

impl Scheduler {
    pub async fn new() -> Result<Self> {
        Self::with_path(DEFAULT_SLOTS_PATH).await
    }

    pub async fn with_path(path: &str) -> Result<Self> {
        let mut slots = BTreeMap::new();
        let mut by_id = HashMap::new();

        for slot in load_slots(path).await? {
            let key = (slot.date, slot.start_time, slot.id);
            by_id.insert(slot.id, key);
            slots.insert(key, Arc::new(slot));
        }

        Ok(Self {
            state: ArcSwap::from_pointee(SchedulerState { slots, by_id }),
            write_lock: Mutex::new(()),
            path: path.to_string(),
        })
    }

    fn get_state(&self) -> Arc<SchedulerState> {
        self.state.load_full()
    }

    pub fn slot_count(&self) -> usize {
        self.get_state().slots.len()
    }

    async fn do_save(&self, state: &SchedulerState) -> Result<()> {
        let slots: Vec<TimeSlot> = state.slots.values().map(|s| s.as_ref().clone()).collect();
        save_slots(&slots, &self.path).await
    }

    fn get_key_by_id(state: &SchedulerState, id: Uuid) -> Option<&SlotKey> {
        state.by_id.get(&id)
    }

    pub async fn create_slot(&self, create_timeslot: CreateTimeSlot) -> Result<TimeSlot> {
        let _guard = self.write_lock.lock().await;

        if create_timeslot.start_time >= create_timeslot.end_time {
            return Err(AppError::InvalidTimeRange);
        }

        let mut new_state = self.get_state().as_ref().clone();
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

        let key = (slot.date, slot.start_time, slot.id);
        new_state.by_id.insert(slot.id, key);
        new_state.slots.insert(key, Arc::new(slot.clone()));

        self.do_save(&new_state).await?;
        self.state.store(Arc::new(new_state));

        info!("Created new slot: {} for date {}", slot.id, slot.date);
        Ok(slot)
    }

    pub async fn book_slot(
        &self,
        slot_id: Uuid,
        create_booking: CreateBooking,
    ) -> Result<TimeSlot> {
        let _guard = self.write_lock.lock().await;

        let mut new_state = self.get_state().as_ref().clone();
        let key = Self::get_key_by_id(&new_state, slot_id)
            .cloned()
            .ok_or(AppError::SlotNotFound)?;

        let slot_arc = new_state
            .slots
            .get_mut(&key)
            .ok_or(AppError::SlotNotFound)?;
        let slot = Arc::make_mut(slot_arc);

        if !slot.is_available {
            return Err(AppError::SlotAlreadyBooked);
        }

        let today = chrono::Utc::now()
            .with_timezone(&chrono_tz::Europe::Moscow)
            .date_naive();
        if slot.date <= today {
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
        let result = slot.clone();

        self.do_save(&new_state).await?;
        self.state.store(Arc::new(new_state));

        info!(
            "Slot {} booked by company: {}",
            slot_id,
            result
                .booking
                .as_ref()
                .map(|b| &b.company_name)
                .unwrap_or(&"Unknown".to_string())
        );
        Ok(result)
    }

    pub fn get_slots_in_range(
        &self,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Vec<TimeSlot> {
        let state = self.get_state();

        let start_bound = from
            .map(|date| Bound::Included((date, NaiveTime::MIN, Uuid::nil())))
            .unwrap_or(Bound::Unbounded);

        // Исключаем начало СЛЕДУЮЩЕГО дня: (дата + 1 день)
        let end_bound = to
            .and_then(|date| date.succ_opt())
            .map(|next_date| Bound::Excluded((next_date, NaiveTime::MIN, Uuid::nil())))
            .unwrap_or(Bound::Unbounded);

        state
            .slots
            .range((start_bound, end_bound))
            .map(|(_, slot)| slot.as_ref().clone())
            .collect()
    }

    pub fn get_slot(&self, id: Uuid) -> Option<TimeSlot> {
        let state = self.get_state();
        let key = Self::get_key_by_id(&state, id)?;
        state.slots.get(key).map(|slot| slot.as_ref().clone())
    }

    pub async fn delete_slot(&self, id: Uuid) -> Result<bool> {
        let _guard = self.write_lock.lock().await;

        let mut new_state = self.get_state().as_ref().clone();

        let key = new_state.by_id.remove(&id).ok_or(AppError::SlotNotFound)?;
        new_state.slots.remove(&key);

        self.do_save(&new_state).await?;
        self.state.store(Arc::new(new_state));

        info!("Slot {} deleted successfully", id);
        Ok(true)
    }

    pub async fn set_yougile_task_id(&self, slot_id: Uuid, task_id: String) -> Result<TimeSlot> {
        let _guard = self.write_lock.lock().await;

        let mut new_state = self.get_state().as_ref().clone();
        let key = Self::get_key_by_id(&new_state, slot_id)
            .cloned()
            .ok_or(AppError::SlotNotFound)?;

        let slot_arc = new_state
            .slots
            .get_mut(&key)
            .ok_or(AppError::SlotNotFound)?;
        let slot = Arc::make_mut(slot_arc);

        slot.yougile_task_id = Some(task_id);
        let result = slot.clone();

        self.do_save(&new_state).await?;
        self.state.store(Arc::new(new_state));

        Ok(result)
    }

    pub async fn update_slot(
        &self,
        id: Uuid,
        update_time_slot: UpdateTimeSlot,
    ) -> Result<TimeSlot> {
        let _guard = self.write_lock.lock().await;

        let mut new_state = self.get_state().as_ref().clone();
        let old_key = new_state.by_id.remove(&id).ok_or(AppError::SlotNotFound)?;

        let mut slot = new_state
            .slots
            .remove(&old_key)
            .ok_or(AppError::SlotNotFound)?
            .as_ref()
            .clone();

        let new_start = update_time_slot.start_time.unwrap_or(slot.start_time);
        let new_end = update_time_slot.end_time.unwrap_or(slot.end_time);
        if new_start >= new_end {
            return Err(AppError::InvalidTimeRange);
        }

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
            if is_available {
                slot.booking = None;
            }
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
        }

        if let Some(completed) = update_time_slot.completed {
            slot.completed = completed;
        }

        let result = slot.clone();
        let new_key = (slot.date, slot.start_time, slot.id);

        new_state.by_id.insert(id, new_key);
        new_state.slots.insert(new_key, Arc::new(slot));

        self.do_save(&new_state).await?;
        self.state.store(Arc::new(new_state));
        info!("Slot {} updated, new data: {:?}", result.id, result);

        Ok(result)
    }
}
