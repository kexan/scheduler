use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{error, info};
use yougile_api_client::YouGileClient;
use yougile_api_client::apis::configuration::Configuration;
use yougile_api_client::models::{CreateTask, UpdateTask};

use crate::error::{AppError, Result};
use crate::scheduler::TimeSlot;
use crate::yougile::config::{self, YougileConfig};
use crate::yougile::models::{BoardInfo, ColumnInfo, ProjectInfo};

pub struct YougileClient {
    inner: Arc<Mutex<YougileInner>>,
}

struct YougileInner {
    client: YouGileClient,
    config: YougileConfig,
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
        let inner = self.inner.lock().await;

        if !inner.config.enabled {
            return None;
        }

        let (title, description) = format_task_data(slot);

        let create_task = CreateTask {
            title,
            column_id: Some(inner.config.column_id.clone()),
            description: Some(description),
            ..Default::default()
        };

        match inner.client.create_task(create_task).await {
            Ok(result) => {
                info!("Created Yougile task {} for slot {}", result.id, slot.id);
                Some(result.id)
            }
            Err(e) => {
                error!("Failed to create Yougile task for slot {}: {}", slot.id, e);
                None
            }
        }
    }

    pub async fn update_task(&self, slot: &TimeSlot) {
        let inner = self.inner.lock().await;

        if !inner.config.enabled {
            return;
        }

        let Some(task_id) = &slot.yougile_task_id else {
            return;
        };

        let (title, description) = format_task_data(slot);

        let update_task = UpdateTask {
            title: Some(title),
            description: Some(description),
            ..Default::default()
        };

        match inner.client.update_task(task_id, update_task).await {
            Ok(_) => {
                info!("Updated Yougile task {} for slot {}", task_id, slot.id);
            }
            Err(e) => {
                error!("Failed to update Yougile task for slot {}: {}", slot.id, e);
            }
        }
    }

    pub async fn delete_task(&self, task_id: &str) {
        let inner = self.inner.lock().await;

        if !inner.config.enabled {
            return;
        }

        let update_task = UpdateTask {
            deleted: Some(true),
            ..Default::default()
        };

        match inner.client.update_task(task_id, update_task).await {
            Ok(_) => {
                info!("Deleted Yougile task {} successfully", task_id);
            }
            Err(e) => {
                error!("Failed to delete Yougile task {}: {}", task_id, e);
            }
        }
    }

    pub async fn load_full_map(&self) -> Result<Vec<ProjectInfo>> {
        let inner = self.inner.lock().await;

        if !inner.config.enabled {
            return Err(AppError::Yougile("Integration disabled".to_string()));
        }

        let projects = inner
            .client
            .search_projects(None, Some(999.0), None, None)
            .await
            .map_err(|e| AppError::Yougile(e.to_string()))?;

        let mut result = Vec::new();

        for project in projects.content {
            let boards = inner
                .client
                .search_boards(None, Some(999.0), None, None, Some(&project.id))
                .await
                .map_err(|e| AppError::Yougile(e.to_string()))?;

            let mut board_infos = Vec::new();
            for board in boards.content {
                let columns = inner
                    .client
                    .search_columns(None, Some(999.0), None, None, Some(&board.id))
                    .await
                    .map_err(|e| AppError::Yougile(e.to_string()))?;

                board_infos.push(BoardInfo {
                    id: board.id,
                    title: board.title,
                    columns: columns
                        .content
                        .into_iter()
                        .map(|c| ColumnInfo {
                            id: c.id,
                            title: c.title,
                        })
                        .collect(),
                });
            }

            result.push(ProjectInfo {
                id: project.id,
                title: project.title,
                boards: board_infos,
            });
        }

        Ok(result)
    }
}

impl YougileInner {
    fn new(config: YougileConfig) -> Result<Self> {
        const YOUGILE_TIMEOUT: Duration = Duration::from_secs(10);

        let cfg = Configuration::new(config.api_token.clone())
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
