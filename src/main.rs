use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use migration_scheduler::error::AppError;
use migration_scheduler::web;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "migration_scheduler=debug,warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Migration Scheduler starting...");

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3030);

    web::start_server(port).await
}
