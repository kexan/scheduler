use crate::api;
use crate::error::Result;
use crate::scheduler::Scheduler;
use crate::yougile::YougileClient;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

pub async fn start_server(port: u16) -> Result<()> {
    info!("Starting web server on port {}", port);

    let scheduler = Arc::new(Scheduler::new().await?);
    let slot_count = scheduler.slot_count();
    if slot_count > 0 {
        info!("Loaded {} slots from storage", slot_count);
    } else {
        info!("Starting with empty scheduler (no data or empty file)");
    }

    let yougile = Arc::new(YougileClient::new().await?);
    info!("Yougile integration initialized");

    let app = api::routes(scheduler, yougile);

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    info!("Server listening at http://127.0.0.1:{}", port);

    axum::serve(listener, app).await?;

    Ok(())
}
