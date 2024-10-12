use crate::task::task_service::task_service;
use std::time::Duration;

pub fn start_worker() {
    tokio::spawn(async move {
        let interval = Duration::from_secs(5);

        loop {
            log::info!("Watching tasks...");
            if let Err(e) = task_service().batch().await {
                log::error!("Error: {:?}", e);
            }
            tokio::time::sleep(interval).await;
        }
    });
}
