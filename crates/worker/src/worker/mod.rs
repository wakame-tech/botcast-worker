use crate::{
    api::Args,
    tasks::{
        episode_repo::{EpisodeRepo, PostgresEpisodeRepo},
        run,
        storage::{R2Storage, Storage},
    },
};
use std::time::Duration;
use task::TaskStatus;
use task_repo::{PostgresTaskRepo, TaskRepo};

pub(crate) mod task;
pub(crate) mod task_repo;

pub fn start_worker() {
    tokio::spawn(async move {
        let interval = Duration::from_secs(5);
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to DB");
        let task_repo = PostgresTaskRepo::new(pool.clone());
        let episode_repo = PostgresEpisodeRepo::new(pool.clone());
        let storage = R2Storage::new().expect("Failed to create storage");

        loop {
            log::info!("Watching tasks...");
            if let Err(e) = batch(&task_repo, &episode_repo, &storage).await {
                log::error!("Error: {:?}", e);
            }
            tokio::time::sleep(interval).await;
        }
    });
}

async fn batch<T: TaskRepo, E: EpisodeRepo, S: Storage>(
    task_repo: &T,
    episode_repo: &E,
    storage: &S,
) -> anyhow::Result<()> {
    let Some(mut task) = task_repo.pop().await? else {
        return Ok(());
    };
    log::info!("Found task: {} args={}", task.id, task.args);
    task.status = TaskStatus::Running;
    task_repo.update(&task).await?;

    let args: Args = serde_json::from_value(task.args.clone())?;
    task.status = match run(episode_repo, storage, task.id, &args).await {
        Ok(()) => TaskStatus::Completed,
        Err(e) => {
            log::error!("Failed to run task: {:?}", e);
            TaskStatus::Failed
        }
    };
    task_repo.update(&task).await?;
    Ok(())
}
