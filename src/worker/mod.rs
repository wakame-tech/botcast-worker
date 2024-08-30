use crate::api::ctx::Ctx;
use std::time::Duration;

pub(crate) mod scrape;
pub(crate) mod synthesis;
pub(crate) mod task;
pub(crate) mod voicevox_client;

pub(crate) fn start_worker() {
    tokio::spawn(async move {
        let ctx = Ctx::new().await.unwrap();
        let interval = Duration::from_secs(5);
        loop {
            log::info!("Watching tasks...");
            let repo = ctx.task_repo();
            if let Err(e) = repo.watch(&ctx).await {
                log::error!("Error: {:?}", e);
            }
            tokio::time::sleep(interval).await;
        }
    });
}
