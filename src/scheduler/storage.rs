use std::path::Path;
use tokio::fs as async_fs;
use tracing::{debug, error};

use crate::error::{AppError, Result};
use crate::scheduler::models::TimeSlot;

const SLOTS_PATH: &str = "data/slots.json";

pub async fn load_slots() -> Result<Vec<TimeSlot>> {
    let content = match async_fs::read_to_string(SLOTS_PATH).await {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            debug!("No data file found, starting fresh");
            return Ok(Vec::new());
        }
        Err(e) => {
            error!("Failed to read {}: {}", SLOTS_PATH, e);
            return Err(AppError::Io(e));
        }
    };

    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    let slots: Vec<TimeSlot> = serde_json::from_str(&content)
        .inspect_err(|e| error!("Failed to parse JSON from {}: {}", SLOTS_PATH, e))
        .map_err(AppError::Json)?;

    Ok(slots)
}

pub async fn save_slots(slots: &[TimeSlot]) -> Result<()> {
    let json = serde_json::to_vec_pretty(slots).map_err(AppError::Json)?;

    let path = Path::new(SLOTS_PATH);
    let tmp_path = path.with_extension("json.tmp");

    if let Some(parent) = path.parent() {
        async_fs::create_dir_all(parent)
            .await
            .map_err(AppError::Io)?;
    }

    async_fs::write(&tmp_path, &json)
        .await
        .map_err(AppError::Io)?;

    async_fs::rename(&tmp_path, path)
        .await
        .map_err(AppError::Io)?;

    Ok(())
}
