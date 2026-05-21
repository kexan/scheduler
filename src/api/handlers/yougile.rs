use crate::error::AppError;
use crate::yougile::models::{BoardInfo, ColumnInfo, ProjectInfo, UserInfo};
use crate::{api::AppState, yougile::YougileConfig};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct YougileConfigResponse {
    pub enabled: bool,
    pub api_url: String,
    pub project_id: Option<String>,
    pub board_id: Option<String>,
    pub column_id: Option<String>,
    pub assignee_id: Option<String>,
}

impl From<YougileConfig> for YougileConfigResponse {
    fn from(config: YougileConfig) -> Self {
        Self {
            enabled: config.enabled,
            api_url: config.api_url,
            project_id: config.project_id,
            board_id: config.board_id,
            column_id: config.column_id,
            assignee_id: config.assignee_id,
        }
    }
}

#[derive(Deserialize)]
pub struct BoardsQuery {
    pub project_id: String,
}

#[derive(Deserialize)]
pub struct ColumnsQuery {
    pub board_id: String,
}

#[derive(Deserialize)]
pub struct UsersQuery {
    pub project_id: String,
}

pub async fn get_yougile_config_handler(
    State(state): State<AppState>,
) -> Json<YougileConfigResponse> {
    Json(state.yougile.config().into())
}

pub async fn update_yougile_config_handler(
    State(state): State<AppState>,
    Json(config): Json<YougileConfig>,
) -> Result<StatusCode, AppError> {
    state.yougile.update_config(config).await?;
    Ok(StatusCode::OK)
}

pub async fn get_projects_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<ProjectInfo>>, AppError> {
    let projects = state.yougile.load_projects().await?;
    Ok(Json(projects))
}

pub async fn get_boards_handler(
    State(state): State<AppState>,
    Query(query): Query<BoardsQuery>,
) -> Result<Json<Vec<BoardInfo>>, AppError> {
    let boards = state.yougile.load_boards(&query.project_id).await?;
    Ok(Json(boards))
}

pub async fn get_columns_handler(
    State(state): State<AppState>,
    Query(query): Query<ColumnsQuery>,
) -> Result<Json<Vec<ColumnInfo>>, AppError> {
    let columns = state.yougile.load_columns(&query.board_id).await?;
    Ok(Json(columns))
}

pub async fn get_users_handler(
    State(state): State<AppState>,
    Query(query): Query<UsersQuery>,
) -> Result<Json<Vec<UserInfo>>, AppError> {
    let users = state.yougile.load_users(&query.project_id).await?;
    Ok(Json(users))
}
