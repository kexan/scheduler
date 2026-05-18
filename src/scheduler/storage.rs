use std::path::Path;
use tokio::fs as async_fs;
use tracing::{debug, error};

use crate::error::{AppError, Result};
use crate::scheduler::models::TimeSlot;
use crate::utils::atomic_write;

pub async fn load_slots(path: &str) -> Result<Vec<TimeSlot>> {
    let content = match async_fs::read_to_string(path).await {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            debug!("No data file found at {}, starting fresh", path);
            return Ok(Vec::new());
        }
        Err(e) => {
            error!("Failed to read {}: {}", path, e);
            return Err(AppError::Io(e));
        }
    };

    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    serde_json::from_str(&content)
        .inspect_err(|e| error!("Failed to parse JSON from {}: {}", path, e))
        .map_err(AppError::Json)
}

pub async fn save_slots(slots: &[TimeSlot], path: &str) -> Result<()> {
    let json = serde_json::to_vec_pretty(slots).map_err(AppError::Json)?;
    atomic_write(Path::new(path), &json).await
}
