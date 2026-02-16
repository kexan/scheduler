use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::fs as async_fs;
use tokio::sync::RwLock;
use tracing::{debug, error, info};
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
    inner: Arc<RwLock<SchedulerInner>>,
}

struct SchedulerInner {
    slots: HashMap<Uuid, TimeSlot>,
}

impl Scheduler {
    pub async fn new() -> Result<Self> {
        let inner = Arc::new(RwLock::new(SchedulerInner {
            slots: HashMap::new(),
        }));

        let content = match async_fs::read_to_string(SLOTS_PATH).await {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                debug!("💾 No data file found, starting fresh");
                return Ok(Self { inner });
            }
            Err(e) => {
                error!("💾 Failed to read {}: {}", SLOTS_PATH, e);
                return Err(AppError::Io(e));
            }
        };

        if content.trim().is_empty() {
            return Ok(Self { inner });
        }

        let slots: Vec<TimeSlot> = serde_json::from_str(&content)
            .inspect_err(|e| error!("💾 Failed to parse JSON from {}: {}", SLOTS_PATH, e))
            .map_err(AppError::Json)?;

        {
            let mut inner = inner.write().await;
            for slot in slots {
                inner.slots.insert(slot.id, slot);
            }
        }

        Ok(Self { inner })
    }

    async fn save(&self) -> Result<()> {
        let slots = self.get_slots().await;
        debug!("💾 Saving {} slots to file {}", slots.len(), SLOTS_PATH);

        let json_content = serde_json::to_string_pretty(&slots)
            .inspect_err(|e| error!("💾 Failed to serialize slots to JSON: {}", e))
            .map_err(AppError::Json)?;

        if let Some(parent) = Path::new(SLOTS_PATH).parent()
            && !parent.exists()
        {
            debug!("📁 Creating directory structure: {}", parent.display());
            async_fs::create_dir_all(parent)
                .await
                .inspect_err(|e| {
                    error!("💾 Failed to create directory {}: {}", parent.display(), e)
                })
                .map_err(AppError::Io)?;
        }

        async_fs::write(SLOTS_PATH, json_content)
            .await
            .inspect_err(|e| error!("💾 Failed to write to file {}: {}", SLOTS_PATH, e))
            .map_err(AppError::Io)?;

        debug!(
            "💾 Successfully saved {} slots to {}",
            slots.len(),
            SLOTS_PATH
        );
        Ok(())
    }

    pub async fn create_slot(&self, request: CreateSlotRequest) -> Result<TimeSlot> {
        let slot = {
            let mut inner = self.inner.write().await;
            let slot = TimeSlot {
                id: Uuid::new_v4(),
                start_time: request.start_time,
                end_time: request.end_time,
                date: request.date,
                is_available: true,
                booking: None,
                yougile_task_id: None,
            };

            inner.slots.insert(slot.id, slot.clone());
            slot
        };
        self.save().await?;

        info!("➕ Created new slot: {} for date {}", slot.id, slot.date);
        Ok(slot)
    }

    pub async fn book_slot(
        &self,
        slot_id: Uuid,
        request: BookingRequest,
        is_admin: bool,
    ) -> Result<TimeSlot> {
        let today = chrono::Local::now().date_naive();

        let slot = {
            let mut inner = self.inner.write().await;
            match inner.slots.get_mut(&slot_id) {
                Some(slot) if slot.is_available || is_admin => {
                    if slot.date <= today && !is_admin {
                        return Err(AppError::Other(
                            "Нельзя записываться на слоты в текущие и прошедшие даты".to_string(),
                        ));
                    }
                    let booking = Booking {
                        company_name: request.company_name,
                        admin_email: request.admin_email,
                        company_id: request.company_id,
                        download_email: request.download_email,
                        created_at: Utc::now(),
                    };

                    slot.is_available = false;
                    slot.booking = Some(booking);
                    slot.clone()
                }
                Some(_) => return Err(AppError::SlotAlreadyBooked),
                None => return Err(AppError::SlotNotFound),
            }
        };

        self.save().await?;
        info!(
            "📝 Slot {} booked by company: {}",
            slot_id,
            slot.booking
                .as_ref()
                .map(|b| &b.company_name)
                .unwrap_or(&"Unknown".to_string())
        );
        Ok(slot)
    }

    pub async fn get_slots(&self) -> Vec<TimeSlot> {
        let inner = self.inner.read().await;
        let mut slots: Vec<_> = inner.slots.values().cloned().collect();
        slots.sort_by(|a, b| a.date.cmp(&b.date).then(a.start_time.cmp(&b.start_time)));
        slots
    }

    pub async fn get_slot(&self, id: Uuid) -> Option<TimeSlot> {
        let inner = self.inner.read().await;
        inner.slots.get(&id).cloned()
    }

    pub async fn delete_slot(&self, id: Uuid) -> Result<Option<TimeSlot>> {
        let removed = {
            let mut inner = self.inner.write().await;
            inner.slots.remove(&id)
        };
        if removed.is_some() {
            self.save().await?;
            info!("🗑️  Slot {} deleted successfully", id);
        }
        Ok(removed)
    }

    pub async fn set_yougile_task_id(&self, slot_id: Uuid, task_id: String) -> Result<TimeSlot> {
        let slot = {
            let mut inner = self.inner.write().await;
            let slot = inner
                .slots
                .get_mut(&slot_id)
                .ok_or(AppError::SlotNotFound)?;

            slot.yougile_task_id = Some(task_id);
            slot.clone()
        };
        self.save().await?;
        Ok(slot)
    }

    pub async fn update_slot(&self, id: Uuid, request: CreateSlotRequest) -> Result<TimeSlot> {
        let slot = {
            let mut inner = self.inner.write().await;
            let slot = inner.slots.get_mut(&id).ok_or(AppError::SlotNotFound)?;

            slot.date = request.date;
            slot.start_time = request.start_time;
            slot.end_time = request.end_time;
            slot.clone()
        };
        self.save().await?;

        info!(
            "✏️  Slot {} updated to date: {}, time: {}-{}",
            slot.id, slot.date, slot.start_time, slot.end_time
        );
        Ok(slot)
    }

    pub async fn update_slot_full(&self, id: Uuid, request: UpdateSlotRequest) -> Result<TimeSlot> {
        let slot = {
            let mut inner = self.inner.write().await;
            let slot = inner.slots.get_mut(&id).ok_or(AppError::SlotNotFound)?;

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
                    created_at: Utc::now(),
                };
                slot.booking = Some(booking);
                slot.is_available = false;
            } else if request.is_available == Some(false) {
                slot.booking = None;
            }

            slot.clone()
        };
        self.save().await?;

        info!(
            "📝 Slot {} fully updated - date: {}, available: {}, has_booking: {}",
            slot.id,
            slot.date,
            slot.is_available,
            slot.booking.is_some()
        );

        Ok(slot)
    }
}
