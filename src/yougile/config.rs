use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs::read_to_string;
use tracing::debug;

use crate::error::{AppError, Result};

const SETTINGS_PATH: &str = "data/yougile_config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YougileConfig {
    pub enabled: bool,
    pub api_url: String,
    pub api_token: Option<String>,
    pub project_id: Option<String>,
    pub board_id: Option<String>,
    pub column_id: Option<String>,
    pub assignee_id: Option<String>,
}

impl Default for YougileConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_url: "https://yougile.com".to_string(),
            api_token: None,
            project_id: None,
            board_id: None,
            column_id: None,
            assignee_id: None,
        }
    }
}

impl YougileConfig {
    pub async fn save(&self) -> Result<()> {
        let json = serde_json::to_vec_pretty(self).map_err(AppError::Json)?;
        crate::utils::atomic_write(Path::new(SETTINGS_PATH), &json).await
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
