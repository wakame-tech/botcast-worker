use super::{Task, TaskRepo, TaskStatus};
use crate::{
    api::Args,
    episode::{episode_service::EpisodeService, script_service::ScriptService},
};
use reqwest::Url;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct TaskService {
    pub(crate) task_repo: Arc<dyn TaskRepo>,
    pub(crate) episode_service: EpisodeService,
    pub(crate) scrape_service: ScriptService,
}

impl TaskService {
    async fn run(&self, task_id: Uuid, episode_id: Uuid, url: Url) -> anyhow::Result<()> {
        // let sentences = self
        //     .scrape_service
        //     .evaluate_to_manuscript(task_id, episode_id, url)
        //     .await?;
        // self.episode_service
        //     .synthesis_audio(task_id, episode_id, sentences)
        //     .await?;
        Ok(())
    }

    async fn run_task(&self, mut task: Task, args: Args) -> anyhow::Result<()> {
        task.status = match self.run(task.id, args.episode_id, args.url.parse()?).await {
            Ok(()) => TaskStatus::Completed,
            Err(e) => {
                log::error!("Failed to run task: {:?}", e);
                TaskStatus::Failed
            }
        };
        self.task_repo.update(&task).await?;
        Ok(())
    }

    pub(crate) async fn batch(&self) -> anyhow::Result<()> {
        let Some(mut task) = self.task_repo.pop().await? else {
            return Ok(());
        };
        log::info!("Found task: {} args={}", task.id, task.args);
        task.status = TaskStatus::Running;
        self.task_repo.update(&task).await?;
        let args: Args = serde_json::from_value(task.args.clone())?;
        self.run_task(task, args).await
    }
}
