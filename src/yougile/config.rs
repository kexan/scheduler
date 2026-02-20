use log::debug;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs::{create_dir_all, read_to_string, write};

use crate::error::{AppError, Result};
use crate::yougile::models::{ProjectInfo, UserInfo};

const SETTINGS_PATH: &str = "data/yougile_config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YougileConfig {
    pub enabled: bool,
    pub api_url: String,
    #[serde(default)]
    pub api_token: String,
    pub project_id: String,
    #[serde(default)]
    pub project_title: String,
    pub board_id: String,
    #[serde(default)]
    pub board_title: String,
    pub column_id: String,
    #[serde(default)]
    pub column_title: String,
    #[serde(default)]
    pub projects_map: Vec<ProjectInfo>,
    #[serde(default)]
    pub users_map: Vec<UserInfo>,
    #[serde(default)]
    pub assignee_id: Option<String>,
    #[serde(default)]
    pub assignee_name: Option<String>,
}

impl Default for YougileConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_url: "https://yougile.com".to_string(),
            api_token: String::new(),
            project_id: String::new(),
            project_title: String::new(),
            board_id: String::new(),
            board_title: String::new(),
            column_id: String::new(),
            column_title: String::new(),
            projects_map: Vec::new(),
            users_map: Vec::new(),
            assignee_id: None,
            assignee_name: None,
        }
    }
}

impl YougileConfig {
    pub async fn save(&self) -> Result<()> {
        let json = serde_json::to_vec_pretty(self).map_err(AppError::Json)?;

        let path = Path::new(SETTINGS_PATH);
        let tmp_path = path.with_extension("json.tmp");

        if let Some(parent) = path.parent() {
            create_dir_all(parent).await.map_err(AppError::Io)?;
        }

        write(&tmp_path, &json).await.map_err(AppError::Io)?;

        tokio::fs::rename(&tmp_path, path)
            .await
            .map_err(AppError::Io)?;

        Ok(())
    }
}

pub async fn load_yougile_settings() -> Result<YougileConfig> {
    let path = Path::new(SETTINGS_PATH);
    if !path.exists() {
        debug!(
            "Yougile settings file {} does not exist, using defaults",
            SETTINGS_PATH
        );
        return Ok(YougileConfig::default());
    }

    let content = read_to_string(SETTINGS_PATH).await.map_err(AppError::Io)?;

    serde_json::from_str(&content).map_err(AppError::Json)
}
