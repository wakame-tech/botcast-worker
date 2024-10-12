use super::{
    model::{Args, Task, TaskStatus},
    repo::{task_repo, TaskRepo},
};
use crate::{
    episode::episode_service::{episode_service, EpisodeService},
    worker::use_work_dir,
};
use std::sync::Arc;
use uuid::Uuid;

pub(crate) fn task_service() -> TaskService {
    TaskService {
        task_repo: task_repo(),
        episode_service: episode_service(),
    }
}

#[derive(Clone)]
pub(crate) struct TaskService {
    task_repo: Arc<dyn TaskRepo>,
    episode_service: EpisodeService,
}

impl TaskService {
    async fn execute(&self, task_id: Uuid, args: Args) -> anyhow::Result<()> {
        match args {
            Args::GenerateAudio { episode_id } => {
                let work_dir = use_work_dir(&task_id)?;
                self.episode_service
                    .generate_audio(&work_dir, episode_id)
                    .await?;
            }
            Args::EvaluateScript { episode_id } => {
                self.episode_service.generate_manuscript(episode_id).await?;
            }
        }
        Ok(())
    }

    async fn run_task(&self, mut task: Task, args: Args) -> anyhow::Result<()> {
        task.status = match self.execute(task.id, args).await {
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
