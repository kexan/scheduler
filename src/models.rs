use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSlot {
    pub id: Uuid,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub date: chrono::NaiveDate,
    pub is_available: bool,
    pub booking: Option<Booking>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Booking {
    pub company_name: String,
    pub admin_email: String,
    pub company_id: String,
    pub download_email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSlotRequest {
    pub date: chrono::NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
}

#[derive(Debug, Serialize, Deserialize)]
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

    pub fn create_slot(&mut self, request: CreateSlotRequest) -> TimeSlot {
        let slot = TimeSlot {
            id: Uuid::new_v4(),
            start_time: request.start_time,
            end_time: request.end_time,
            date: request.date,
            is_available: true,
            booking: None,
        };

        self.slots.insert(slot.id, slot.clone());
        slot
    }

    pub fn book_slot(
        &mut self,
        slot_id: Uuid,
        request: BookingRequest,
    ) -> Result<TimeSlot, String> {
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
            Some(_) => Err("Слот уже занят".to_string()),
            None => Err("Слот не найден".to_string()),
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

    pub fn update_slot(
        &mut self,
        id: Uuid,
        request: CreateSlotRequest,
    ) -> Result<TimeSlot, String> {
        match self.slots.get_mut(&id) {
            Some(slot) => {
                slot.date = request.date;
                slot.start_time = request.start_time;
                slot.end_time = request.end_time;
                Ok(slot.clone())
            }
            None => Err("Слот не найден".to_string()),
        }
    }
}
