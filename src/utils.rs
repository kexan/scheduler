use std::{path::Path, time::Duration};
use tokio::{fs as async_fs, time::sleep};
use tracing::{error, warn};

use crate::error::{AppError, IsFatalError, Result};

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

const MAX_RETRIES: u32 = 15;

pub async fn with_retry<F, Fut, T, E>(
    operation: &str,
    entity_id: &str,
    mut f: F,
) -> std::result::Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = std::result::Result<T, E>>,
    E: std::fmt::Display + IsFatalError,
{
    let mut attempts = 0;
    loop {
        attempts += 1;
        match f().await {
            Ok(res) => return Ok(res),
            Err(e) => {
                if e.is_fatal() {
                    warn!(
                        "Fatal error encountered during {} for {}: {}. Dropping retry loop.",
                        operation, entity_id, e
                    );
                    return Err(e);
                }

                if attempts < MAX_RETRIES {
                    let delay = calc_backoff_delay(attempts - 1);
                    error!(
                        "Retry {}/{} for {} failed ({}): {}. Retrying in {:?}",
                        attempts, MAX_RETRIES, operation, entity_id, e, delay
                    );
                    sleep(delay).await;
                } else {
                    error!("Failed to {} after {} attempts: {}", operation, attempts, e);
                    return Err(e);
                }
            }
        }
    }
}

fn calc_backoff_delay(attempt: u32) -> Duration {
    let base = 1000u64;
    let max = 2 * 60 * 60 * 1000;
    let delay = base * 2u64.pow(attempt);
    let delay = delay.min(max);
    let jitter = (fastrand::u64(0..1000) as f64 / 1000.0 * delay as f64) as u64;
    Duration::from_millis(delay + jitter)
}
