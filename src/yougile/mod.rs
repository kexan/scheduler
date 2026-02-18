pub mod config;

use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, error, info};
use yougile_api_client::YouGileClient;
use yougile_api_client::apis::configuration::Configuration;
use yougile_api_client::models::{CreateTask, UpdateTask};

use crate::error::AppError;
use crate::scheduler::TimeSlot;
use crate::yougile::config::*;

#[derive(Error, Debug)]
pub enum YougileError {
    #[error("Integration disabled")]
    Disabled,

    #[error("Client not initialized")]
    NotInitialized,

    #[error("API error: {0}")]
    Api(String),
}

impl From<YougileError> for AppError {
    fn from(e: YougileError) -> Self {
        AppError::Yougile(e.to_string())
    }
}

pub struct YougileIntegration {
    inner: Arc<RwLock<YougileInner>>,
}

struct YougileInner {
    client: Option<YouGileClient>,
    settings: YougileSettings,
}

impl YougileIntegration {
    pub async fn new() -> Result<Self, YougileError> {
        let settings = load_yougile_settings()
            .await
            .map_err(|e| YougileError::Api(e.to_string()))?;

        let inner = Arc::new(RwLock::new(YougileInner::new(settings)));

        Ok(Self { inner })
    }

    pub async fn update_settings(&self, settings: YougileSettings) -> Result<(), YougileError> {
        settings
            .save()
            .await
            .map_err(|e| YougileError::Api(e.to_string()))?;

        let mut inner = self.inner.write().await;
        *inner = YougileInner::new(settings);
        Ok(())
    }

    pub async fn settings(&self) -> YougileSettings {
        self.inner.read().await.settings.clone()
    }

    pub async fn is_enabled(&self) -> bool {
        self.inner.read().await.is_enabled()
    }

    pub async fn create_task(&self, slot: &TimeSlot) -> Option<String> {
        let state = self.inner.read().await;

        if !state.is_enabled() {
            info!("Yougile integration disabled, skipping task creation");
            return None;
        }

        match state.create_migration_task(slot).await {
            Ok(task_id) => {
                info!("Created Yougile task: {} for slot {}", task_id, slot.id);
                Some(task_id)
            }
            Err(e) => {
                error!("Failed to create Yougile task for slot {}: {}", slot.id, e);
                None
            }
        }
    }

    pub async fn update_task(&self, slot: &TimeSlot) {
        let state = self.inner.read().await;

        if !state.is_enabled() {
            debug!("Yougile integration disabled, skipping task update");
            return;
        }

        match state.update_migration_task(slot).await {
            Ok(_) => {
                info!("Updated Yougile task for slot {}", slot.id);
            }
            Err(e) => {
                error!("Failed to update Yougile task for slot {}: {}", slot.id, e);
            }
        }
    }

    pub async fn delete_task(&self, task_id: &str) {
        let state = self.inner.read().await;

        if !state.is_enabled() {
            debug!("Yougile integration disabled, skipping task deletion");
            return;
        }

        match state.delete_migration_task(task_id).await {
            Ok(_) => {
                info!("Deleted Yougile task: {}", task_id);
            }
            Err(e) => {
                error!("Failed to delete Yougile task {}: {}", task_id, e);
            }
        }
    }

    pub async fn load_full_map(&self) -> Result<Vec<ProjectInfo>, YougileError> {
        let state = self.inner.read().await;

        if !state.is_enabled() {
            debug!("Yougile integration disabled, skipping loading");
            return Err(YougileError::Disabled);
        }

        state.load_full_map().await
    }
}

impl YougileInner {
    fn new(settings: YougileSettings) -> Self {
        let client = if settings.enabled && !settings.api_token.is_empty() {
            let config =
                Configuration::new(settings.api_token.clone()).with_base_path(&settings.api_url);
            Some(YouGileClient::new(config))
        } else {
            None
        };

        Self { client, settings }
    }

    fn is_enabled(&self) -> bool {
        self.settings.enabled && self.client.is_some()
    }

    async fn create_migration_task(&self, slot: &TimeSlot) -> Result<String, YougileError> {
        let client = self.client.as_ref().ok_or(YougileError::NotInitialized)?;

        let task_title = self.create_task_name(slot);
        let task_description = self.create_task_description(slot);

        let create_task = CreateTask {
            title: task_title,
            column_id: Some(self.settings.column_id.clone()),
            description: Some(task_description),
            ..Default::default()
        };

        let result = client
            .create_task(create_task)
            .await
            .map_err(|e| YougileError::Api(e.to_string()))?;
        Ok(result.id)
    }

    async fn update_migration_task(&self, slot: &TimeSlot) -> Result<(), YougileError> {
        let task_id = match &slot.yougile_task_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let client = self.client.as_ref().ok_or(YougileError::NotInitialized)?;

        let task_title = self.create_task_name(slot);
        let task_description = self.create_task_description(slot);

        let update_task = UpdateTask {
            title: Some(task_title),
            description: Some(task_description),
            ..Default::default()
        };

        client
            .update_task(task_id, update_task)
            .await
            .map_err(|e| YougileError::Api(e.to_string()))?;

        Ok(())
    }

    async fn delete_migration_task(&self, task_id: &str) -> Result<(), YougileError> {
        let client = self.client.as_ref().ok_or(YougileError::NotInitialized)?;

        let update_task = UpdateTask {
            deleted: Some(true),
            ..Default::default()
        };

        client
            .update_task(task_id, update_task)
            .await
            .map_err(|e| YougileError::Api(e.to_string()))?;

        Ok(())
    }

    fn create_task_description(&self, slot: &TimeSlot) -> String {
        if let Some(booking) = &slot.booking {
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
        }
    }

    fn create_task_name(&self, slot: &TimeSlot) -> String {
        format!(
            "Миграция: {}",
            slot.booking
                .as_ref()
                .map(|b| &b.company_name)
                .unwrap_or(&"Неизвестная компания".to_string())
        )
    }

    async fn get_projects(&self) -> Result<Vec<yougile_api_client::models::Project>, YougileError> {
        if !self.is_enabled() {
            return Err(YougileError::Disabled);
        }

        let client = self.client.as_ref().ok_or(YougileError::NotInitialized)?;

        let projects = client
            .search_projects(None, Some(999.0), None, None)
            .await
            .map_err(|e| YougileError::Api(e.to_string()))?;

        Ok(projects.content)
    }

    async fn get_boards(
        &self,
        project_id: &str,
    ) -> Result<Vec<yougile_api_client::models::Board>, YougileError> {
        if !self.is_enabled() {
            return Err(YougileError::Disabled);
        }

        let client = self.client.as_ref().ok_or(YougileError::NotInitialized)?;

        let boards = client
            .search_boards(None, Some(999.0), None, None, Some(project_id))
            .await
            .map_err(|e| YougileError::Api(e.to_string()))?;

        Ok(boards.content)
    }

    async fn get_columns(
        &self,
        board_id: &str,
    ) -> Result<Vec<yougile_api_client::models::Column>, YougileError> {
        if !self.is_enabled() {
            return Err(YougileError::Disabled);
        }

        let client = self.client.as_ref().ok_or(YougileError::NotInitialized)?;

        let columns = client
            .search_columns(None, Some(999.0), None, None, Some(board_id))
            .await
            .map_err(|e| YougileError::Api(e.to_string()))?;

        Ok(columns.content)
    }

    async fn load_full_map(&self) -> Result<Vec<ProjectInfo>, YougileError> {
        let projects = self.get_projects().await?;

        let mut projects_map = Vec::new();

        for project in projects {
            let mut project_info = ProjectInfo {
                id: project.id.clone(),
                title: project.title.clone(),
                boards: Vec::new(),
            };

            let boards = self.get_boards(&project.id).await?;
            for board in boards {
                let mut board_info = BoardInfo {
                    id: board.id.clone(),
                    title: board.title.clone(),
                    columns: Vec::new(),
                };

                let columns = self.get_columns(&board.id).await?;
                for column in columns {
                    board_info.columns.push(ColumnInfo {
                        id: column.id.clone(),
                        title: column.title.clone(),
                    });
                }

                project_info.boards.push(board_info);
            }

            projects_map.push(project_info);
        }

        Ok(projects_map)
    }
}
