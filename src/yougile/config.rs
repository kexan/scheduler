use crate::error::{AppError, Result};
use log::debug;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs::{create_dir_all, read_to_string, write};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardInfo {
    pub id: String,
    pub title: String,
    pub columns: Vec<ColumnInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: String,
    pub title: String,
    pub boards: Vec<BoardInfo>,
}

const SETTINGS_PATH: &str = "yougile_settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YougileSettings {
    pub enabled: bool,
    pub api_url: String,
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
}

impl Default for YougileSettings {
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
        }
    }
}

impl YougileSettings {
    pub async fn save(&self) -> Result<()> {
        let json = serde_json::to_vec_pretty(self).map_err(AppError::Json)?;

        let path = Path::new(SETTINGS_PATH);
        let tmp_path = path.with_extension("json.tmp");

        if let Some(parent) = path.parent() {
            create_dir_all(parent).await.map_err(AppError::Io)?;
        }

        write(&tmp_path, &json).await.map_err(AppError::Io)?;

        write(&tmp_path, &json).await.map_err(AppError::Io)?;

        tokio::fs::rename(&tmp_path, path)
            .await
            .map_err(AppError::Io)?;

        Ok(())
    }
}

pub async fn load_yougile_settings() -> Result<YougileSettings> {
    let path = Path::new(SETTINGS_PATH);
    if !path.exists() {
        debug!(
            "Yougile settings file {} does not exist, using defaults",
            SETTINGS_PATH
        );
        return Ok(YougileSettings::default());
    }

    let content = read_to_string(SETTINGS_PATH).await.map_err(AppError::Io)?;

    serde_json::from_str(&content).map_err(AppError::Json)
}
