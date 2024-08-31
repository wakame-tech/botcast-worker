use crate::{api::ctx::Ctx, model::Task, repo::TaskRepo};
use scrape::ScrapeEpisode;
use std::{fmt::Debug, time::Duration};
use synthesis::Synthesis;
use uuid::Uuid;

pub(crate) mod scrape;
pub(crate) mod synthesis;
pub(crate) mod voicevox_client;

pub(crate) trait RunTask
where
    Self: Debug + Clone + serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    async fn run(&self, id: Uuid, ctx: &Ctx) -> anyhow::Result<Option<Args>>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) enum Args {
    Scrape(ScrapeEpisode),
    Synthesis(Synthesis),
}

impl Args {
    async fn run(self, id: Uuid, ctx: &Ctx) -> anyhow::Result<Option<Args>> {
        match self {
            Self::Scrape(scrape) => scrape.run(id, ctx).await,
            Self::Synthesis(synthesis) => synthesis.run(id, ctx).await,
        }
    }
}

pub(crate) fn start_worker() {
    tokio::spawn(async move {
        let ctx = Ctx::new().await.unwrap();
        let interval = Duration::from_secs(5);
        loop {
            log::info!("Watching tasks...");
            let repo = TaskRepo::new(ctx.pool.clone());
            if let Err(e) = batch(repo, &ctx).await {
                log::error!("Error: {:?}", e);
            }
            tokio::time::sleep(interval).await;
        }
    });
}

async fn batch(repo: TaskRepo, ctx: &Ctx) -> anyhow::Result<()> {
    let mut tasks = repo.list().await?;
    if tasks.is_empty() {
        return Ok(());
    }
    log::info!("worker | {} tasks", tasks.len());
    for task in tasks.iter_mut() {
        let Task { id, args, .. } = &task;
        let args: Args = serde_json::from_value(args.clone())?;
        match args.run(task.id, &ctx).await {
            Ok(Some(args)) => {
                let task = Task::new(args)?;
                repo.create(task).await?;
            }
            Err(e) => {
                log::error!("Failed to run task: {:?}", e);
            }
            _ => {}
        };
        let res = repo.delete(&id).await?;
        log::info!("Deleted: {:?}", res.id);
    }
    Ok(())
}
