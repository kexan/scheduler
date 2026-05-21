use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs::read_to_string;
use tracing::debug;

use crate::error::{AppError, Result};

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
    pub async fn save(&self, path: &str) -> Result<()> {
        let json = serde_json::to_vec_pretty(self).map_err(AppError::Json)?;
        crate::utils::atomic_write(Path::new(path), &json).await
    }
}

pub async fn load_yougile_config(path: &str) -> Result<YougileConfig> {
    let content = match read_to_string(path).await {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            debug!(
                "Yougile settings file {} does not exist, using defaults",
                path
            );
            return Ok(YougileConfig::default());
        }
        Err(e) => return Err(AppError::Io(e)),
    };

    serde_json::from_str(&content).map_err(AppError::Json)
}
