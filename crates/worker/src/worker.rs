use crate::app_module::AppModule;
use std::time::Duration;

pub fn start_worker(app_module: AppModule) {
    tokio::spawn(async move {
        let interval = Duration::from_secs(5);

        loop {
            log::info!("Watching tasks...");
            if let Err(e) = app_module.task_service.batch().await {
                log::error!("Error: {:?}", e);
            }
            tokio::time::sleep(interval).await;
        }
    });
}
