use crate::task::task_service::task_service;
use audio_generator::workdir::WorkDir;
use std::time::Duration;
use uuid::Uuid;

pub(crate) fn use_work_dir(task_id: &Uuid) -> anyhow::Result<WorkDir> {
    let keep = std::env::var("KEEP_WORKDIR")
        .unwrap_or("false".to_string())
        .parse()?;
    WorkDir::new(task_id, keep)
}

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
