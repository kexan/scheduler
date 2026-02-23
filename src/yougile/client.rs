use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{error, info};
use yougile_api_client::YouGileClient;
use yougile_api_client::apis::configuration::Configuration;
use yougile_api_client::models::tasks::UpdateDeadline;
use yougile_api_client::models::{CreateTask, Deadline, UpdateTask};

use crate::error::{AppError, Result};
use crate::scheduler::TimeSlot;
use crate::yougile::config::{self, YougileConfig};
use crate::yougile::models::{BoardInfo, ColumnInfo, ProjectInfo, UserInfo};

pub struct YougileClient {
    inner: Arc<Mutex<YougileInner>>,
}

struct YougileInner {
    client: YouGileClient,
    config: YougileConfig,
}

const MAX_RETRIES: u32 = 15;

fn calc_backoff_delay(attempt: u32) -> Duration {
    let base = 1000_u64;
    let max = 2 * 60 * 60 * 1000; // 2 hours
    let delay = base * 2u64.pow(attempt);
    let delay = delay.min(max);
    let jitter = (fastrand::u64(0..1000) as f64 / 1000.0 * delay as f64) as u64;
    Duration::from_millis(delay + jitter)
}

impl YougileClient {
    pub async fn new() -> Result<Self> {
        let config = config::load_yougile_settings().await?;
        let inner = Arc::new(Mutex::new(YougileInner::new(config)?));
        Ok(Self { inner })
    }

    pub async fn update_config(&self, settings: YougileConfig) -> Result<()> {
        let new_inner = YougileInner::new(settings.clone())?;

        settings.save().await?;

        let mut inner = self.inner.lock().await;
        *inner = new_inner;

        Ok(())
    }

    pub async fn config(&self) -> YougileConfig {
        self.inner.lock().await.config.clone()
    }

    pub async fn is_enabled(&self) -> bool {
        self.inner.lock().await.config.enabled
    }

    pub async fn create_task(&self, slot: &TimeSlot) -> Option<String> {
        let slot_id = slot.id.to_string();
        let (title, description) = format_task_data(slot);
        let mut attempts = 0;

        loop {
            let (client, column_id, enabled, assignee) = {
                let inner = self.inner.lock().await;
                if !inner.config.enabled {
                    return None;
                }
                (
                    inner.client.clone(),
                    inner.config.column_id.clone().unwrap_or_default(),
                    inner.config.enabled,
                    inner.config.assignee_id.clone(),
                )
            };

            if !enabled {
                return None;
            }

            let assigned = assignee.map(|id| vec![id]);
            let deadline = build_deadline(slot);

            let create_task = CreateTask {
                title: title.clone(),
                column_id: if column_id.is_empty() {
                    None
                } else {
                    Some(column_id)
                },
                description: Some(description.clone()),
                assigned,
                deadline: Some(deadline),
                ..Default::default()
            };

            attempts += 1;
            match client.create_task(create_task).await {
                Ok(res) => {
                    info!("Created Yougile task {} for slot {}", res.id, slot_id);
                    return Some(res.id);
                }
                Err(e) if attempts < MAX_RETRIES => {
                    let delay = calc_backoff_delay(attempts - 1);
                    error!(
                        "Retry {}/{} for create_task failed (slot_id={}): {}. Retrying in {:?}",
                        attempts, MAX_RETRIES, slot_id, e, delay
                    );
                    sleep(delay).await;
                }
                Err(e) => {
                    error!(
                        "Failed to create Yougile task for slot {} after {} attempts: {}",
                        slot_id, attempts, e
                    );
                    return None;
                }
            }
        }
    }

    pub async fn update_task(&self, slot: &TimeSlot) {
        let task_id = match &slot.yougile_task_id {
            Some(id) => id.clone(),
            None => return,
        };
        let slot_id = slot.id.to_string();
        let (title, description) = format_task_data(slot);
        let mut attempts = 0;

        loop {
            let (client, enabled, assignee) = {
                let inner = self.inner.lock().await;
                if !inner.config.enabled {
                    return;
                }
                (
                    inner.client.clone(),
                    inner.config.enabled,
                    inner.config.assignee_id.clone(),
                )
            };

            if !enabled {
                return;
            }

            let assigned = assignee.map(|id| vec![id]);
            let deadline = build_update_deadline(slot);

            let update_task = UpdateTask {
                title: Some(title.clone()),
                description: Some(description.clone()),
                completed: Some(slot.completed),
                assigned,
                deadline: Some(deadline),
                ..Default::default()
            };

            attempts += 1;
            match client.update_task(&task_id, update_task).await {
                Ok(_) => {
                    info!("Updated Yougile task {} for slot {}", task_id, slot_id);
                    return;
                }
                Err(e) if attempts < MAX_RETRIES => {
                    let delay = calc_backoff_delay(attempts - 1);
                    error!(
                        "Retry {}/{} for update_task failed (task_id={}, slot_id={}): {}. Retrying in {:?}",
                        attempts, MAX_RETRIES, task_id, slot_id, e, delay
                    );
                    sleep(delay).await;
                }
                Err(e) => {
                    error!(
                        "Failed to update Yougile task {} for slot {} after {} attempts: {}",
                        task_id, slot_id, attempts, e
                    );
                    return;
                }
            }
        }
    }

    pub async fn delete_task(&self, task_id: &str) {
        let task_id = task_id.to_string();
        let mut attempts = 0;

        loop {
            let (client, enabled) = {
                let inner = self.inner.lock().await;
                if !inner.config.enabled {
                    return;
                }
                (inner.client.clone(), inner.config.enabled)
            };

            if !enabled {
                return;
            }

            let update_task = UpdateTask {
                deleted: Some(true),
                ..Default::default()
            };

            attempts += 1;
            match client.update_task(&task_id, update_task).await {
                Ok(_) => {
                    info!("Deleted Yougile task {} successfully", task_id);
                    return;
                }
                Err(e) if attempts < MAX_RETRIES => {
                    let delay = calc_backoff_delay(attempts - 1);
                    error!(
                        "Retry {}/{} for delete_task failed (task_id={}): {}. Retrying in {:?}",
                        attempts, MAX_RETRIES, task_id, e, delay
                    );
                    sleep(delay).await;
                }
                Err(e) => {
                    error!(
                        "Failed to delete Yougile task {} after {} attempts: {}",
                        task_id, attempts, e
                    );
                    return;
                }
            }
        }
    }

    pub async fn load_projects(&self) -> Result<Vec<ProjectInfo>> {
        let inner = self.inner.lock().await;

        if !inner.config.enabled {
            return Err(AppError::Yougile("Integration disabled".to_string()));
        }

        let projects = inner
            .client
            .search_projects(None, Some(999.0), None, None)
            .await
            .map_err(|e| AppError::Yougile(e.to_string()))?;

        Ok(projects
            .content
            .into_iter()
            .map(|p| ProjectInfo {
                id: p.id,
                title: p.title,
            })
            .collect())
    }

    pub async fn load_boards(&self, project_id: &str) -> Result<Vec<BoardInfo>> {
        let inner = self.inner.lock().await;

        if !inner.config.enabled {
            return Err(AppError::Yougile("Integration disabled".to_string()));
        }

        let boards = inner
            .client
            .search_boards(None, Some(999.0), None, None, Some(project_id))
            .await
            .map_err(|e| AppError::Yougile(e.to_string()))?;

        Ok(boards
            .content
            .into_iter()
            .map(|b| BoardInfo {
                id: b.id,
                title: b.title,
            })
            .collect())
    }

    pub async fn load_columns(&self, board_id: &str) -> Result<Vec<ColumnInfo>> {
        let inner = self.inner.lock().await;

        if !inner.config.enabled {
            return Err(AppError::Yougile("Integration disabled".to_string()));
        }

        let columns = inner
            .client
            .search_columns(None, Some(999.0), None, None, Some(board_id))
            .await
            .map_err(|e| AppError::Yougile(e.to_string()))?;

        Ok(columns
            .content
            .into_iter()
            .map(|c| ColumnInfo {
                id: c.id,
                title: c.title,
            })
            .collect())
    }

    pub async fn load_users(&self, project_id: &str) -> Result<Vec<UserInfo>> {
        let inner = self.inner.lock().await;

        if !inner.config.enabled {
            return Err(AppError::Yougile("Integration disabled".to_string()));
        }

        let users = inner
            .client
            .search_users(Some(999.0), None, None, Some(project_id))
            .await
            .map_err(|e| AppError::Yougile(e.to_string()))?;

        let result = users
            .content
            .into_iter()
            .map(|u| UserInfo {
                id: u.id,
                email: u.email,
                name: u.real_name,
            })
            .collect();

        Ok(result)
    }
}

impl YougileInner {
    fn new(config: YougileConfig) -> Result<Self> {
        const YOUGILE_TIMEOUT: Duration = Duration::from_secs(10);

        let cfg = Configuration::new(config.api_token.clone().unwrap_or_default())
            .with_base_path(&config.api_url)
            .with_timeout(YOUGILE_TIMEOUT);
        let client = YouGileClient::new(cfg);
        Ok(Self { client, config })
    }
}

fn format_task_data(slot: &TimeSlot) -> (String, String) {
    let title = format!(
        "Миграция: {}",
        slot.booking
            .as_ref()
            .map(|b| b.company_name.as_str())
            .unwrap_or("Неизвестная компания")
    );

    let description = if let Some(booking) = &slot.booking {
        format!(
            "Информация о бронировании:<br/>\
            • Компания: {}<br/>\
            • Email администратора: {}<br/>\
            • ID компании: {}<br/>\
            • Email получателя архива: {}<br/>\
            • Дата: {}<br/>\
            • Время: {} - {}<br/>\
            • Создано: {}",
            booking.company_name,
            booking.admin_email,
            booking.company_id,
            booking.download_email,
            slot.date,
            slot.start_time.format("%H:%M"),
            slot.end_time.format("%H:%M"),
            booking.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        )
    } else {
        format!(
            "Информация о слоте:<br/>\
            • Дата: {}<br/>\
            • Время: {} - {}<br/>\
            • Статус: Доступен",
            slot.date,
            slot.start_time.format("%H:%M"),
            slot.end_time.format("%H:%M")
        )
    };

    (title, description)
}

//FIXME: костыльно чето
const MOSCOW_OFFSET_SECONDS: i32 = 3 * 60 * 60;
use chrono::TimeZone;

/// Returns `(start_timestamp_ms, end_timestamp_ms)` in Moscow time as f64.
fn slot_to_timestamps(slot: &TimeSlot) -> (f64, f64) {
    let moscow_offset = chrono::FixedOffset::east_opt(MOSCOW_OFFSET_SECONDS).unwrap();

    let start_dt = moscow_offset
        .from_local_datetime(&slot.date.and_time(slot.start_time))
        .single()
        .unwrap();
    let end_dt = moscow_offset
        .from_local_datetime(&slot.date.and_time(slot.end_time))
        .single()
        .unwrap();

    (
        start_dt.timestamp_millis() as f64,
        end_dt.timestamp_millis() as f64,
    )
}

fn build_deadline(slot: &TimeSlot) -> Deadline {
    let (start_timestamp, end_timestamp) = slot_to_timestamps(slot);
    Deadline {
        deadline: end_timestamp,
        start_date: Some(start_timestamp),
        with_time: Some(true),
        ..Default::default()
    }
}

fn build_update_deadline(slot: &TimeSlot) -> UpdateDeadline {
    let (start_timestamp, end_timestamp) = slot_to_timestamps(slot);
    UpdateDeadline {
        deadline: Some(end_timestamp),
        start_date: Some(start_timestamp),
        with_time: Some(true),
        ..Default::default()
    }
}
