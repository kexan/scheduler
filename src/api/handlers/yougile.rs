use crate::api::handlers::YougileSettings;
use crate::error::AppError;
use crate::yougile::YougileIntegration;
use std::sync::Arc;
use tracing::{error, info};
use warp::Rejection;

pub async fn get_yougile_settings_handler(
    yougile: Arc<YougileIntegration>,
) -> Result<warp::reply::Json, Rejection> {
    let settings = yougile.settings().await;
    Ok(warp::reply::json(&settings))
}

pub async fn update_yougile_settings_handler(
    settings: YougileSettings,
    yougile: Arc<YougileIntegration>,
) -> Result<warp::reply::Json, Rejection> {
    yougile
        .update_settings(settings.clone())
        .await
        .map_err(|e| warp::reject::custom(AppError::Yougile(e.to_string())))?;

    info!("Yougile settings updated successfully");
    Ok(warp::reply::json(&serde_json::json!({
        "success": true,
        "settings": settings
    })))
}

pub async fn test_yougile_connection_handler(
    yougile: Arc<YougileIntegration>,
) -> Result<warp::reply::Json, Rejection> {
    let projects_map = yougile.load_full_map().await.map_err(|e| {
        error!("Yougile connection test failed: {}", e);
        warp::reject::custom(AppError::Yougile(e.to_string()))
    })?;

    info!(
        "Yougile connection test successful, loaded {} projects",
        projects_map.len()
    );

    Ok(warp::reply::json(&serde_json::json!({
        "success": true,
        "message": format!("Connection successful, loaded {} projects", projects_map.len()),
        "projects_map": projects_map
    })))
}
