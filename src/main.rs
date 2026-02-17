mod api;
mod cli;
mod error;
mod scheduler;
mod web;
mod yougile;

use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "migration_scheduler=debug,warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Logging initialized");
}

use error::AppError;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    init_logging();

    info!("Migration Scheduler starting...");

    cli::run_cli().await
}
