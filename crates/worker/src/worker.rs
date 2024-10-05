use crate::{
    api::Args,
    tasks::{
        episode_repo::PostgresEpisodeRepo,
        storage::R2Storage,
        task::{Task, TaskStatus},
        task_repo::{PostgresTaskRepo, TaskRepo},
        EpisodeService,
    },
};
use std::{sync::Arc, time::Duration};

pub fn start_worker() {
    tokio::spawn(async move {
        let interval = Duration::from_secs(5);
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to DB");
        let episode_repo = PostgresEpisodeRepo::new(pool.clone());
        let storage = R2Storage::new().expect("Failed to create storage");
        let episode_service = Arc::new(EpisodeService {
            episode_repo: Box::new(episode_repo),
            storage: Box::new(storage),
        });

        let task_repo = PostgresTaskRepo::new(pool.clone());
        let task_service = Arc::new(RunTaskService {
            task_repo: Box::new(task_repo),
            episode_service: episode_service.clone(),
        });

        loop {
            log::info!("Watching tasks...");
            if let Err(e) = task_service.batch().await {
                log::error!("Error: {:?}", e);
            }
            tokio::time::sleep(interval).await;
        }
    });
}

struct RunTaskService {
    task_repo: Box<dyn TaskRepo>,
    episode_service: Arc<EpisodeService>,
}

impl RunTaskService {
    async fn run_task(&self, mut task: Task, args: Args) -> anyhow::Result<()> {
        task.status = match self
            .episode_service
            .run(task.id, args.episode_id, args.url.parse()?)
            .await
        {
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
