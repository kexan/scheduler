use crate::api;
use crate::error::Result;
use crate::scheduler::Scheduler;
use crate::yougile::YougileClient;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

pub async fn start_server(host: &str, port: u16) -> Result<()> {
    info!("Starting web server on {}:{}", host, port);

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

    let listener = TcpListener::bind(format!("{}:{}", host, port)).await?;
    info!("Server listening at http://{}:{}", host, port);

    axum::serve(listener, app).await?;

    Ok(())
}
