mod models;
mod handlers;

use std::sync::Arc;
use tokio::sync::RwLock;
use models::Scheduler;

#[tokio::main]
async fn main() {
    let scheduler = Arc::new(RwLock::new(Scheduler::new()));
    
    let routes = handlers::routes(scheduler);
    
    println!("🚀 Сервер запущен на http://localhost:3030");
    println!("📅 Откройте http://localhost:3030 в браузере");
    
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
