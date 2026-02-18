use crate::api;
use crate::error::Result;
use crate::scheduler::Scheduler;
use crate::yougile::YougileClient;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

pub async fn start_server(scheduler: Scheduler, port: u16) -> Result<()> {
    info!("Starting web server on port {}", port);

    let slot_count = scheduler.get_slots().await.len();
    if slot_count > 0 {
        info!("Successfully loaded scheduler with {} slots", slot_count);
    } else {
        info!("Starting with empty scheduler (no data or empty file)");
    }

    let scheduler = Arc::new(scheduler);
    info!("Scheduler initialized");

    let yougile = Arc::new(YougileClient::new().await?);
    info!("Yougile integration initialized");

    let app = api::routes(scheduler, yougile);

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    info!("Server starting at http://localhost:{}", port);

    axum::serve(listener, app).await?;

    Ok(())
}
