use crate::{api::Args, episode_repo::PostgresEpisodeRepo, tasks::run};
use sqlx::{Pool, Postgres};
use std::time::Duration;
use task::TaskStatus;
use task_repo::{PostgresTaskRepo, TaskRepo};

pub(crate) mod task;
pub(crate) mod task_repo;

pub fn start_worker() {
    tokio::spawn(async move {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to DB");
        let interval = Duration::from_secs(5);
        loop {
            log::info!("Watching tasks...");
            if let Err(e) = batch(pool.clone()).await {
                log::error!("Error: {:?}", e);
            }
            tokio::time::sleep(interval).await;
        }
    });
}

async fn batch(pool: Pool<Postgres>) -> anyhow::Result<()> {
    let mut task_repo = PostgresTaskRepo::new(pool.clone());
    let Some(mut task) = task_repo.pop().await? else {
        return Ok(());
    };

    let mut episode_repo = PostgresEpisodeRepo::new(pool);
    task.status = TaskStatus::Running;
    task_repo.update(&task).await?;

    let args: Args = serde_json::from_value(task.args.clone())?;
    task.status = match run(&mut episode_repo, task.id, &args).await {
        Ok(()) => TaskStatus::Completed,
        Err(e) => {
            log::error!("Failed to run task: {:?}", e);
            TaskStatus::Failed
        }
    };
    task_repo.update(&task).await?;
    Ok(())
}
