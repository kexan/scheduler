use std::path::Path;
use tokio::fs as async_fs;

use crate::error::{AppError, Result};

pub async fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    let tmp_path = path.with_extension("json.tmp");

    if let Some(parent) = path.parent() {
        async_fs::create_dir_all(parent)
            .await
            .map_err(AppError::Io)?;
    }

    async_fs::write(&tmp_path, data)
        .await
        .map_err(AppError::Io)?;

    async_fs::rename(&tmp_path, path)
        .await
        .map_err(AppError::Io)?;

    Ok(())
}
