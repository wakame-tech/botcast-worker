use super::episode_service::EpisodeService;
use super::script_service::ScriptService;
use crate::error::Error;
use crate::{model::Args, worker::use_work_dir};
use chrono::{DateTime, Utc};
use repos::entity::{Task, TaskStatus};
use repos::repo::TaskRepo;
use std::collections::BTreeMap;
use std::sync::Arc;
use uuid::Uuid;

pub(crate) fn new_task(args: Args, execute_after: DateTime<Utc>) -> Task {
    Task {
        id: Uuid::new_v4(),
        status: TaskStatus::Pending,
        args: serde_json::to_value(args).unwrap(),
        result: None,
        execute_after,
        executed_at: None,
    }
}

#[derive(Clone)]
pub(crate) struct TaskService {
    task_repo: Arc<dyn TaskRepo>,
    episode_service: EpisodeService,
    script_service: ScriptService,
}

impl TaskService {
    pub(crate) fn new(
        task_repo: Arc<dyn TaskRepo>,
        episode_service: EpisodeService,
        script_service: ScriptService,
    ) -> Self {
        Self {
            task_repo,
            episode_service,
            script_service,
        }
    }

    async fn execute(&self, task: &Task) -> anyhow::Result<(), Error> {
        let args: Args = serde_json::from_value(task.args.clone())
            .map_err(|e| Error::InvalidInput(anyhow::anyhow!("Args {}", e)))?;
        match args {
            Args::GenerateAudio { episode_id } => {
                let work_dir = use_work_dir(&task.id).map_err(|e| {
                    Error::Other(anyhow::anyhow!("Failed to create work dir: {}", e))
                })?;
                self.episode_service
                    .generate_audio(&work_dir, &episode_id)
                    .await?;
            }
            Args::EvaluateScript { script_id } => {
                self.script_service
                    .evaluate_script(&script_id, BTreeMap::new())
                    .await?;
            }
            Args::NewEpisode { podcast_id } => {
                let Some(task) = self
                    .episode_service
                    .new_episode_from_template(&podcast_id)
                    .await?
                else {
                    return Ok(());
                };
                self.task_repo.create(&task).await?;
            }
        }
        Ok(())
    }

    async fn run_task(&self, mut task: Task) -> anyhow::Result<(), Error> {
        task.status = TaskStatus::Running;
        self.task_repo.update(&task).await?;
        task.status = match self.execute(&task).await {
            Ok(()) => TaskStatus::Completed,
            Err(e) => {
                log::error!("Failed to run task: {:?}", e);
                TaskStatus::Failed
            }
        };
        task.executed_at = Some(Utc::now());
        self.task_repo.update(&task).await?;
        log::info!("task: {} completed", task.id);
        Ok(())
    }

    pub(crate) async fn create_task(&self, args: Args) -> anyhow::Result<(), Error> {
        let task = new_task(args, Utc::now());
        self.task_repo.create(&task).await?;
        Ok(())
    }

    pub(crate) async fn execute_queued_tasks(&self) -> anyhow::Result<(), Error> {
        let Some(task) = self.task_repo.pop(Utc::now()).await? else {
            return Ok(());
        };
        log::info!("Found task: {} args={}", task.id, task.args);
        self.run_task(task).await?;
        Ok(())
    }
}
