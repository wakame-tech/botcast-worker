use super::episode_service::EpisodeService;
use super::script_service::ScriptService;
use crate::error::Error;
use crate::worker::use_work_dir;
use anyhow::Context;
use api::client::ApiClient;
use chrono::{DateTime, Utc};
use repos::entity::{EpisodeId, Task, TaskStatus};
use repos::repo::TaskRepo;
use std::collections::BTreeMap;
use std::str::FromStr;
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub(crate) enum Args {
    GenerateAudio {
        episode_id: EpisodeId,
    },
    EvaluateTemplate {
        template: serde_json::Value,
        context: BTreeMap<String, serde_json::Value>,
    },
}

pub(crate) fn new_task(
    user_id: Option<Uuid>,
    cron: Option<String>,
    args: Args,
    execute_after: DateTime<Utc>,
) -> Task {
    Task {
        id: Uuid::new_v4(),
        user_id,
        status: TaskStatus::Pending,
        cron,
        args: serde_json::to_value(args).unwrap(),
        result: None,
        execute_after,
        executed_at: None,
    }
}

#[derive(Clone)]
pub(crate) struct TaskService {
    task_repo: Arc<dyn TaskRepo>,
    api_client: Arc<ApiClient>,
    episode_service: EpisodeService,
    script_service: ScriptService,
}

impl TaskService {
    pub(crate) fn new(
        task_repo: Arc<dyn TaskRepo>,
        api_client: Arc<ApiClient>,
        episode_service: EpisodeService,
        script_service: ScriptService,
    ) -> Self {
        Self {
            task_repo,
            api_client,
            episode_service,
            script_service,
        }
    }

    #[instrument(skip(self))]
    async fn execute(&self, task: &Task) -> anyhow::Result<serde_json::Value, Error> {
        let args: Args = serde_json::from_value(task.args.clone())
            .map_err(|e| Error::InvalidInput(anyhow::anyhow!("Args {}", e)))?;

        if let Some(cron) = &task.cron {
            let next = cron::Schedule::from_str(cron)
                .context("Invalid cron")
                .map_err(Error::Other)?
                .upcoming(Utc)
                .next()
                .context("Failed to get next cron")
                .map_err(Error::Other)?;
            let task = new_task(task.user_id, Some(cron.to_string()), args.clone(), next);
            self.task_repo.create(&task).await?;
        }

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
                let result = self.script_service.run_template(&template, context).await?;
                Ok(result)
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
        let user = self
            .api_client
            .me()
            .await
            .context("Failed to get user")
            .map_err(Error::Other)?;
        let user_id = user
            .id
            .parse()
            .context("Failed to parse user id")
            .map_err(Error::Other)?;
        let task = new_task(Some(user_id), None, args, Utc::now());
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
