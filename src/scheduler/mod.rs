use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs as async_fs;
use tracing::{debug, error};
use uuid::Uuid;

use crate::api::handlers::slots::{CreateSlotRequest, UpdateSlotRequest};
use crate::error::{AppError, Result};

const SLOTS_PATH: &str = "data/slots.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSlot {
    pub id: Uuid,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub date: chrono::NaiveDate,
    pub is_available: bool,
    pub booking: Option<Booking>,
    pub yougile_task_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Booking {
    pub company_name: String,
    pub admin_email: String,
    pub company_id: String,
    pub download_email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookingRequest {
    pub company_name: String,
    pub admin_email: String,
    pub company_id: String,
    pub download_email: String,
}

pub struct Scheduler {
    slots: HashMap<Uuid, TimeSlot>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            slots: HashMap::new(),
        }
    }

    pub async fn load() -> Result<Self> {
        let path = Path::new(SLOTS_PATH);
        if !path.exists() {
            debug!("💾 Data file {} does not exist, starting empty", SLOTS_PATH);
            return Ok(Self::new());
        }

        let content = async_fs::read_to_string(SLOTS_PATH).await.map_err(|e| {
            error!("💾 Failed to read data file {}: {}", SLOTS_PATH, e);
            AppError::Io(e)
        })?;

        if content.trim().is_empty() {
            debug!("💾 Data file {} is empty", SLOTS_PATH);
            return Ok(Self::new());
        }

        let slots: Vec<TimeSlot> = serde_json::from_str(&content).map_err(|e| {
            error!("💾 Failed to parse JSON from file {}: {}", SLOTS_PATH, e);
            AppError::Json(e)
        })?;

        Ok(Self::from_slots(slots))
    }

    pub async fn save(&self) -> Result<()> {
        let slots = self.get_slots();
        debug!("💾 Saving {} slots to file {}", slots.len(), SLOTS_PATH);

        let json_content = serde_json::to_string_pretty(&slots).map_err(|e| {
            error!("💾 Failed to serialize slots to JSON: {}", e);
            AppError::Json(e)
        })?;

        if let Some(parent) = Path::new(SLOTS_PATH).parent()
            && !parent.exists()
        {
            debug!("📁 Creating directory structure: {}", parent.display());
            async_fs::create_dir_all(parent).await.map_err(|e| {
                error!("💾 Failed to create directory {}: {}", parent.display(), e);
                AppError::Io(e)
            })?;
        }

        async_fs::write(SLOTS_PATH, json_content)
            .await
            .map_err(|e| {
                error!("💾 Failed to write to file {}: {}", SLOTS_PATH, e);
                AppError::Io(e)
            })?;

        debug!(
            "💾 Successfully saved {} slots to {}",
            slots.len(),
            SLOTS_PATH
        );
        Ok(())
    }

    pub fn from_slots(slots: Vec<TimeSlot>) -> Self {
        let mut scheduler = Self::new();
        for slot in slots {
            scheduler.slots.insert(slot.id, slot);
        }
        scheduler
    }

    pub fn create_slot(&mut self, request: CreateSlotRequest) -> TimeSlot {
        let slot = TimeSlot {
            id: Uuid::new_v4(),
            start_time: request.start_time,
            end_time: request.end_time,
            date: request.date,
            is_available: true,
            booking: None,
            yougile_task_id: None,
        };

        self.slots.insert(slot.id, slot.clone());
        slot
    }

    pub fn book_slot(&mut self, slot_id: Uuid, request: BookingRequest) -> Result<TimeSlot> {
        match self.slots.get_mut(&slot_id) {
            Some(slot) if slot.is_available => {
                let booking = Booking {
                    company_name: request.company_name,
                    admin_email: request.admin_email,
                    company_id: request.company_id,
                    download_email: request.download_email,
                    created_at: Utc::now(),
                };

                slot.is_available = false;
                slot.booking = Some(booking.clone());

                Ok(slot.clone())
            }
            Some(_) => Err(AppError::SlotAlreadyBooked),
            None => Err(AppError::SlotNotFound),
        }
    }

    pub fn get_slots(&self) -> Vec<TimeSlot> {
        let mut slots: Vec<_> = self.slots.values().cloned().collect();
        slots.sort_by(|a, b| a.date.cmp(&b.date).then(a.start_time.cmp(&b.start_time)));
        slots
    }

    pub fn get_slot(&self, id: Uuid) -> Option<&TimeSlot> {
        self.slots.get(&id)
    }

    pub fn delete_slot(&mut self, id: Uuid) -> bool {
        self.slots.remove(&id).is_some()
    }

    pub fn set_yougile_task_id(&mut self, slot_id: Uuid, task_id: String) -> Option<TimeSlot> {
        if let Some(slot) = self.slots.get_mut(&slot_id) {
            slot.yougile_task_id = Some(task_id);
            return Some(slot.clone());
        }
        None
    }

    pub fn update_slot(&mut self, id: Uuid, request: CreateSlotRequest) -> Result<TimeSlot> {
        match self.slots.get_mut(&id) {
            Some(slot) => {
                slot.date = request.date;
                slot.start_time = request.start_time;
                slot.end_time = request.end_time;
                Ok(slot.clone())
            }
            None => Err(AppError::SlotNotFound),
        }
    }

    pub fn update_slot_full(&mut self, id: Uuid, request: UpdateSlotRequest) -> Result<TimeSlot> {
        match self.slots.get_mut(&id) {
            Some(slot) => {
                slot.date = request.date;
                slot.start_time = request.start_time;
                slot.end_time = request.end_time;

                if let Some(is_available) = request.is_available {
                    slot.is_available = is_available;
                }

                if let Some(partial_booking) = request.booking {
                    let booking = Booking {
                        company_name: partial_booking.company_name,
                        admin_email: partial_booking.admin_email,
                        company_id: partial_booking.company_id,
                        download_email: partial_booking.download_email,
                        created_at: chrono::Utc::now(),
                    };
                    slot.booking = Some(booking);
                    slot.is_available = false;
                } else if request.is_available == Some(false) {
                    slot.booking = None;
                }

                Ok(slot.clone())
            }
            None => Err(AppError::SlotNotFound),
        }
    }
}
