use crate::{
    api::Args,
    tasks::{episode_repo::PostgresEpisodeRepo, storage::R2Storage, EpisodeService},
};
use std::{sync::Arc, time::Duration};
use task::TaskStatus;
use task_repo::{PostgresTaskRepo, TaskRepo};
use uuid::Uuid;

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
        let episode_repo = PostgresEpisodeRepo::new(pool);
        let storage = R2Storage::new().expect("Failed to create storage");
        let episode_service = Arc::new(EpisodeService {
            episode_repo: Box::new(episode_repo),
            storage: Box::new(storage),
        });

        loop {
            log::info!("Watching tasks...");
            if let Err(e) = batch(task_repo.clone(), episode_service.clone()).await {
                log::error!("Error: {:?}", e);
            }
            tokio::time::sleep(interval).await;
        }
    });
}

async fn batch(
    task_repo: impl TaskRepo,
    episode_service: Arc<EpisodeService>,
) -> anyhow::Result<()> {
    let Some(mut task) = task_repo.pop().await? else {
        return Ok(());
    };
    log::info!("Found task: {} args={}", task.id, task.args);
    task.status = TaskStatus::Running;
    task_repo.update(&task).await?;

    let args: Args = serde_json::from_value(task.args.clone())?;
    task.status = match run(episode_service.as_ref(), task.id, &args).await {
        Ok(()) => TaskStatus::Completed,
        Err(e) => {
            log::error!("Failed to run task: {:?}", e);
            TaskStatus::Failed
        }
    };
    task_repo.update(&task).await?;
    Ok(())
}

pub(crate) async fn run(
    episode_service: &EpisodeService,
    task_id: Uuid,
    args: &Args,
) -> anyhow::Result<()> {
    let episode_id = Uuid::parse_str(&args.episode_id)?;
    let sentences = episode_service
        .generate_script_from_url(task_id, episode_id, args.url.parse()?)
        .await?;
    episode_service
        .synthesis_audio(task_id, episode_id, sentences)
        .await?;
    Ok(())
}
