use crate::error::AppError;
use crate::yougile::models::ProjectInfo;
use crate::{api::AppState, yougile::YougileConfig};
use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YougileConfigResponse {
    pub enabled: bool,
    pub api_url: String,
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

impl From<YougileConfig> for YougileConfigResponse {
    fn from(config: YougileConfig) -> Self {
        Self {
            enabled: config.enabled,
            api_url: config.api_url,
            project_id: config.project_id,
            project_title: config.project_title,
            board_id: config.board_id,
            board_title: config.board_title,
            column_id: config.column_id,
            column_title: config.column_title,
            projects_map: config.projects_map,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct YougileConfigUpdate {
    pub enabled: Option<bool>,
    pub api_url: Option<String>,
    pub api_token: Option<String>,
    pub project_id: Option<String>,
    pub project_title: Option<String>,
    pub board_id: Option<String>,
    pub board_title: Option<String>,
    pub column_id: Option<String>,
    pub column_title: Option<String>,
}

pub async fn get_yougile_config_handler(
    State(state): State<AppState>,
) -> Result<Json<YougileConfigResponse>, AppError> {
    let config = state.yougile.config().await;
    Ok(Json(config.into()))
}

pub async fn update_yougile_config_handler(
    State(state): State<AppState>,
    Json(update): Json<YougileConfigUpdate>,
) -> Result<StatusCode, AppError> {
    let mut config = state.yougile.config().await;

    if let Some(enabled) = update.enabled {
        config.enabled = enabled;
    }
    if let Some(api_url) = update.api_url {
        config.api_url = api_url;
    }
    if let Some(api_token) = update.api_token {
        config.api_token = api_token;
    }
    if let Some(project_id) = update.project_id {
        config.project_id = project_id;
    }
    if let Some(project_title) = update.project_title {
        config.project_title = project_title;
    }
    if let Some(board_id) = update.board_id {
        config.board_id = board_id;
    }
    if let Some(board_title) = update.board_title {
        config.board_title = board_title;
    }
    if let Some(column_id) = update.column_id {
        config.column_id = column_id;
    }
    if let Some(column_title) = update.column_title {
        config.column_title = column_title;
    }

    state.yougile.update_config(config).await?;
    info!("Yougile settings updated successfully");
    Ok(StatusCode::OK)
}

pub async fn test_yougile_connection_handler(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let projects_map = state.yougile.load_full_map().await.map_err(|e| {
        error!("Yougile connection test failed: {}", e);
        AppError::Yougile(e.to_string())
    })?;

    info!(
        "Yougile connection test successful, loaded {} projects",
        projects_map.len()
    );

    //FIXME: надо на фронте переделать чтобы ожидалась просто projects_map, без всяких success
    Ok(Json(json!({
        "success": true,
        "message": format!("Connection successful, loaded {} projects", projects_map.len()),
        "projects_map": projects_map
    })))
}
