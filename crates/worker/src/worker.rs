use crate::usecase::provider::Provider;
use audio_generator::workdir::WorkDir;
use std::{sync::Arc, time::Duration};
use uuid::Uuid;

pub(crate) fn use_work_dir(task_id: &Uuid) -> anyhow::Result<WorkDir> {
    let keep = std::env::var("KEEP_WORKDIR")
        .unwrap_or("false".to_string())
        .parse()?;
    WorkDir::new(task_id, keep)
}

pub fn start_worker(provider: Arc<Provider>) {
    tokio::spawn(async move {
        let task_service = provider.task_service();
        let interval = Duration::from_secs(5);

        loop {
            // tracing::info!("Watching tasks...");

            if let Err(e) = task_service.execute_queued_tasks().await {
                tracing::error!("Error: {:?}", e);
            }
            tokio::time::sleep(interval).await;
        }
    });
}
