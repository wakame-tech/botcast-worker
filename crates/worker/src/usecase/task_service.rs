use super::episode_service::EpisodeService;
use super::script_service::ScriptService;
use crate::error::Error;
use crate::{model::Args, worker::use_work_dir};
use anyhow::Context;
use chrono::{DateTime, Utc};
use repos::entity::{Task, TaskStatus};
use repos::repo::TaskRepo;
use std::sync::Arc;
use tracing::instrument;
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
    service_role_key: String,
}

impl TaskService {
    pub(crate) fn new(
        task_repo: Arc<dyn TaskRepo>,
        episode_service: EpisodeService,
        script_service: ScriptService,
    ) -> Self {
        let service_role_key = std::env::var("SUPABASE_SERVICE_ROLE_KEY")
            .expect("SUPABASE_SERVICE_ROLE_KEY is not set");
        Self {
            task_repo,
            episode_service,
            script_service,
            service_role_key,
        }
    }

    #[instrument(skip(self))]
    async fn execute(&self, task: &Task) -> anyhow::Result<serde_json::Value, Error> {
        let args: Args = serde_json::from_value(task.args.clone())
            .map_err(|e| Error::InvalidInput(anyhow::anyhow!("Args {}", e)))?;
        match args {
            Args::GenerateAudio { episode_id } => {
                let work_dir = use_work_dir(&task.id)
                    .context("Failed to create work dir")
                    .map_err(Error::Other)?;
                self.episode_service
                    .generate_audio(&work_dir, &episode_id)
                    .await?;
                Ok(serde_json::Value::String("OK".to_string()))
            }
            Args::EvaluateTemplate { template, context } => {
                let result = self
                    .script_service
                    .run_template(self.service_role_key.clone(), &template, context)
                    .await?;
                Ok(result)
            }
            Args::NewEpisode { podcast_id } => {
                if let Some(next_task) = self
                    .episode_service
                    .new_episode_from_template(self.service_role_key.clone(), &podcast_id)
                    .await?
                {
                    self.task_repo.create(&next_task).await?;
                };
                Ok(serde_json::Value::String("OK".to_string()))
            }
        }
    }

    async fn run_task(&self, mut task: Task) -> anyhow::Result<(), Error> {
        task.status = TaskStatus::Running;
        self.task_repo.update(&task).await?;
        (task.status, task.result) = match self.execute(&task).await {
            Ok(result) => (TaskStatus::Completed, Some(result)),
            Err(e) => (
                TaskStatus::Failed,
                Some(serde_json::Value::String(e.to_string())),
            ),
        };
        task.executed_at = Some(Utc::now());
        self.task_repo.update(&task).await?;
        tracing::info!("task: {} completed", task.id);
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
        tracing::info!("Found task: {} args={}", task.id, task.args);
        self.run_task(task).await?;
        Ok(())
    }
}
