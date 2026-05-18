pub mod config;
pub mod models;

pub use config::YougileConfig;
pub use config::load_yougile_config;

use arc_swap::ArcSwap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::info;
use yougile_api_client::{
    YouGileClient as YougileApiClient,
    apis::configuration::Configuration,
    models::{CreateTask, Deadline, UpdateTask, tasks::UpdateDeadline},
};

use crate::scheduler::TimeSlot;
use crate::yougile::models::{BoardInfo, ColumnInfo, ProjectInfo, UserInfo};
use crate::{
    error::{AppError, Result},
    utils::with_retry,
};

const DEFAULT_CONFIG_PATH: &str = "data/yougile_config.json";

pub struct YougileClient {
    state: ArcSwap<YougileState>,
    write_lock: Mutex<()>,
    path: String,
}

struct YougileState {
    api: YougileApiClient,
    config: YougileConfig,
}

fn build_api_client(config: &YougileConfig) -> YougileApiClient {
    const YOUGILE_TIMEOUT: Duration = Duration::from_secs(10);
    let cfg = Configuration::new(config.api_token.clone().unwrap_or_default())
        .with_base_path(&config.api_url)
        .with_timeout(YOUGILE_TIMEOUT);
    YougileApiClient::new(cfg)
}

impl YougileClient {
    pub async fn new() -> Result<Self> {
        Self::with_path(DEFAULT_CONFIG_PATH).await
    }

    pub async fn with_path(path: &str) -> Result<Self> {
        let config = config::load_yougile_config(path).await?;
        Self::with_config(config, path).await
    }

    pub async fn with_config(config: YougileConfig, path: &str) -> Result<Self> {
        let api = build_api_client(&config);
        Ok(Self {
            state: ArcSwap::from_pointee(YougileState { api, config }),
            write_lock: Mutex::new(()),
            path: path.to_string(),
        })
    }

    fn get_state(&self) -> Arc<YougileState> {
        self.state.load_full()
    }

    pub async fn update_config(&self, mut new_config: YougileConfig) -> Result<()> {
        let _guard = self.write_lock.lock().await;

        if new_config.api_token.is_none() {
            new_config.api_token = self.get_state().config.api_token.clone();
        }

        new_config.save(&self.path).await?;

        let api = build_api_client(&new_config);
        self.state.store(Arc::new(YougileState {
            api,
            config: new_config,
        }));

        info!("Yougile settings updated successfully");

        Ok(())
    }

    pub fn config(&self) -> YougileConfig {
        self.get_state().config.clone()
    }

    fn skip_if_disabled(&self, state: &YougileState, action: &str) -> bool {
        if !state.config.enabled {
            info!("Yougile integration disabled, skipping {}", action);
            true
        } else {
            false
        }
    }

    pub async fn create_task(&self, slot: &TimeSlot) -> Result<String> {
        let state = self.get_state();

        if self.skip_if_disabled(&state, &format!("task creation for slot {}", slot.id)) {
            return Ok(String::new());
        }

        let config = &state.config;

        let (start, end) = slot.to_timestamps_ms();
        let deadline = Deadline {
            deadline: end,
            start_date: Some(start),
            with_time: Some(true),
            ..Default::default()
        };

        let new_task = CreateTask {
            title: slot.build_task_title(),
            column_id: config.column_id.clone().filter(|s| !s.is_empty()),
            description: Some(slot.build_task_description()),
            assigned: config.assignee_id.clone().map(|id| vec![id]),
            deadline: Some(deadline),
            ..Default::default()
        };

        let slot_id = slot.id.to_string();

        with_retry("create_task", &slot_id, || async {
            state
                .api
                .create_task(new_task.clone())
                .await
                .map_err(|e| AppError::Yougile(e.to_string()))
        })
        .await
        .map(|res| {
            info!("Created Yougile task {} for slot {}", res.id, slot_id);
            res.id
        })
    }

    pub async fn update_task(&self, slot: &TimeSlot) -> Result<()> {
        let state = self.get_state();

        if self.skip_if_disabled(&state, &format!("task update for slot {}", slot.id)) {
            return Ok(());
        }

        let task_id = match &slot.yougile_task_id {
            Some(id) => id.clone(),
            None => {
                return Err(AppError::Yougile(format!(
                    "Slot {} has no Yougile task ID to update",
                    slot.id
                )));
            }
        };

        let config = &state.config;

        let (start, end) = slot.to_timestamps_ms();
        let deadline = UpdateDeadline {
            deadline: Some(end),
            start_date: Some(start),
            with_time: Some(true),
            ..Default::default()
        };

        let task = UpdateTask {
            title: Some(slot.build_task_title()),
            description: Some(slot.build_task_description()),
            completed: Some(slot.completed),
            assigned: config.assignee_id.clone().map(|id| vec![id]),
            deadline: Some(deadline),
            ..Default::default()
        };

        with_retry("update_task", &task_id, || async {
            state
                .api
                .update_task(&task_id, task.clone())
                .await
                .map_err(|e| AppError::Yougile(e.to_string()))
        })
        .await
        .map(|id| {
            info!("Updated Yougile task {} for slot {}", id.id, slot.id);
        })
    }

    pub async fn delete_task(&self, task_id: &str) -> Result<()> {
        let state = self.get_state();

        if self.skip_if_disabled(&state, &format!("task deletion for {}", task_id)) {
            return Ok(());
        }

        let task = UpdateTask {
            deleted: Some(true),
            ..Default::default()
        };

        with_retry("delete_task", task_id, || async {
            state
                .api
                .update_task(task_id, task.clone())
                .await
                .map_err(|e| AppError::Yougile(e.to_string()))
        })
        .await
        .map(|id| {
            info!("Deleted Yougile task {} successfully", id.id);
        })
    }

    pub async fn load_projects(&self) -> Result<Vec<ProjectInfo>> {
        let state = self.get_state();

        if self.skip_if_disabled(&state, "load projects") {
            return Ok(Vec::new());
        }

        let projects = state
            .api
            .search_projects(None, Some(999.0), None, None)
            .await
            .map_err(|e| AppError::Yougile(e.to_string()))?;

        let result: Vec<ProjectInfo> = projects
            .content
            .into_iter()
            .map(|p| ProjectInfo {
                id: p.id,
                title: p.title,
            })
            .collect();

        info!("Loaded {} Yougile projects", result.len());

        Ok(result)
    }

    pub async fn load_boards(&self, project_id: &str) -> Result<Vec<BoardInfo>> {
        let state = self.get_state();

        if self.skip_if_disabled(&state, &format!("load boards for project {}", project_id)) {
            return Ok(Vec::new());
        }

        let boards = state
            .api
            .search_boards(None, Some(999.0), None, None, Some(project_id))
            .await
            .map_err(|e| AppError::Yougile(e.to_string()))?;

        let result: Vec<BoardInfo> = boards
            .content
            .into_iter()
            .map(|b| BoardInfo {
                id: b.id,
                title: b.title,
            })
            .collect();

        info!("Loaded {} Yougile boards for project {}", result.len(), project_id);

        Ok(result)
    }

    pub async fn load_columns(&self, board_id: &str) -> Result<Vec<ColumnInfo>> {
        let state = self.get_state();

        if self.skip_if_disabled(&state, &format!("load columns for board {}", board_id)) {
            return Ok(Vec::new());
        }

        let columns = state
            .api
            .search_columns(None, Some(999.0), None, None, Some(board_id))
            .await
            .map_err(|e| AppError::Yougile(e.to_string()))?;

        let result: Vec<ColumnInfo> = columns
            .content
            .into_iter()
            .map(|c| ColumnInfo {
                id: c.id,
                title: c.title,
            })
            .collect();

        info!("Loaded {} Yougile columns for board {}", result.len(), board_id);

        Ok(result)
    }

    pub async fn load_users(&self, project_id: &str) -> Result<Vec<UserInfo>> {
        let state = self.get_state();

        if self.skip_if_disabled(&state, &format!("load users for project {}", project_id)) {
            return Ok(Vec::new());
        }

        let users = state
            .api
            .search_users(Some(999.0), None, None, Some(project_id))
            .await
            .map_err(|e| AppError::Yougile(e.to_string()))?;

        let result: Vec<UserInfo> = users
            .content
            .into_iter()
            .map(|u| UserInfo {
                id: u.id,
                email: u.email,
                name: u.real_name,
            })
            .collect();

        info!("Loaded {} Yougile users for project {}", result.len(), project_id);

        Ok(result)
    }
}
