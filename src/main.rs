use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use migration_scheduler::cli;
use migration_scheduler::error::AppError;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "migration_scheduler=debug,warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Logging initialized");
    info!("Migration Scheduler starting...");

    cli::run_cli().await
}
