use crate::{
    episode::script_service::script_service,
    task::{
        model::{Args, Task, TaskStatus},
        repo::{task_repo, TaskRepo},
    },
};
use crate::{
    episode::{
        episode_service::{episode_service, EpisodeService},
        script_service::ScriptService,
    },
    worker::use_work_dir,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use super::repo::TaskId;

pub(crate) fn task_service() -> TaskService {
    TaskService {
        task_repo: task_repo(),
        episode_service: episode_service(),
        script_service: script_service(),
    }
}

#[derive(Clone)]
pub(crate) struct TaskService {
    task_repo: Arc<dyn TaskRepo>,
    episode_service: EpisodeService,
    script_service: ScriptService,
}

impl TaskService {
    async fn execute(&self, task_id: &TaskId, args: Args) -> anyhow::Result<()> {
        match args {
            Args::GenerateAudio { episode_id } => {
                let work_dir = use_work_dir(&task_id.0)?;
                self.episode_service
                    .generate_audio(&work_dir, &episode_id)
                    .await?;
            }
            Args::EvaluateScript { script_id } => {
                self.script_service.evaluate_script(&script_id).await?;
            }
            Args::NewEpisode { pre_episode_id } => {
                let task = self.episode_service.new_episode(&pre_episode_id).await?;
                self.task_repo.create(&task).await?;
            }
        }
        Ok(())
    }

    async fn run_task(&self, task: &mut Task) -> anyhow::Result<()> {
        let args: Args = serde_json::from_value(task.args.clone())?;
        task.status = match self.execute(&TaskId(task.id), args).await {
            Ok(()) => TaskStatus::Completed,
            Err(e) => {
                log::error!("Failed to run task: {:?}", e);
                TaskStatus::Failed
            }
        };
        task.executed_at = Some(Utc::now());
        self.task_repo.update(&task).await?;
        Ok(())
    }

    pub(crate) async fn insert_task(&self, args: serde_json::Value) -> anyhow::Result<()> {
        let args: Args = serde_json::from_value(args)?;
        let task = Task {
            id: Uuid::new_v4(),
            status: TaskStatus::Pending,
            args: serde_json::to_value(args)?,
            execute_after: Utc::now(),
            executed_at: None,
        };
        self.task_repo.create(&task).await?;
        Ok(())
    }

    pub(crate) async fn batch(&self) -> anyhow::Result<()> {
        let Some(mut task) = self.task_repo.pop().await? else {
            return Ok(());
        };
        log::info!("Found task: {} args={}", task.id, task.args);
        task.status = TaskStatus::Running;
        self.task_repo.update(&task).await?;
        self.run_task(&mut task).await?;
        log::info!("task: {} completed", task.id);
        Ok(())
    }
}
