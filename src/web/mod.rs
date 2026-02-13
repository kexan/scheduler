use crate::api;
use crate::error::Result;
use crate::scheduler::Scheduler;
use include_dir::{Dir, include_dir};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use warp::{Rejection, Reply};

pub async fn start_server(port: u16) -> Result<()> {
    info!("🚀 Starting web server on port {}", port);

    let scheduler = match Scheduler::load().await {
        Ok(sched) => {
            info!(
                "📂 Successfully loaded scheduler with {} slots",
                sched.get_slots().len()
            );
            sched
        }
        Err(e) => {
            warn!(
                "⚠️  Error loading scheduler: {}. Starting with empty list",
                e
            );
            Scheduler::new()
        }
    };

    let scheduler = Arc::new(RwLock::new(scheduler));
    info!("📅 Scheduler initialized");

    let routes = api::routes(scheduler);

    let addr = ([127, 0, 0, 1], port);
    info!("🌐 Server starting at http://localhost:{}", port);

    warp::serve(routes).run(addr).await;

    Ok(())
}

const STATIC_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/static");

pub async fn static_handler(
    path: warp::path::Tail,
) -> std::result::Result<warp::reply::Response, Rejection> {
    let path = path.as_str();

    if path.starts_with("api/") {
        return Err(warp::reject::not_found());
    }

    let file = STATIC_DIR
        .get_file(path)
        .or_else(|| STATIC_DIR.get_file("index.html"));

    match file {
        Some(file) => {
            let actual_path = if path.is_empty() || STATIC_DIR.get_file(path).is_none() {
                "index.html"
            } else {
                path
            };

            let content_type = match Path::new(actual_path)
                .extension()
                .and_then(|ext| ext.to_str())
            {
                Some("html") => "text/html",
                Some("css") => "text/css",
                Some("js") => "application/javascript",
                _ => "application/octet-stream",
            };
            let response = warp::reply::with_header(file.contents(), "content-type", content_type);
            Ok(response.into_response())
        }
        None => Err(warp::reject::not_found()),
    }
}
