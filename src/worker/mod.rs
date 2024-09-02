use crate::{
    api::ctx::Ctx,
    model::{Task, TaskStatus},
    repo::TaskRepo,
};
use scrape::Scrape;
use std::{fmt::Debug, time::Duration};
use synthesis::Synthesis;
use uuid::Uuid;

pub(crate) mod extractor;
pub mod r2_client;
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
#[serde(tag = "type")]
pub(crate) enum Args {
    Scrape(Scrape),
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

pub fn start_worker() {
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
    let Some(mut task) = repo.pop().await? else {
        return Ok(());
    };
    task.status = TaskStatus::Running;
    repo.update_status(&task).await?;

    let args: Args = serde_json::from_value(task.args.clone())?;
    match args.run(task.id, &ctx).await {
        Ok(args) => {
            task.status = TaskStatus::Completed;
            if let Some(args) = args {
                let task = Task::new(args)?;
                repo.create(task).await?;
            }
        }
        Err(e) => {
            task.status = TaskStatus::Failed;
            log::error!("Failed to run task: {:?}", e);
        }
    };
    repo.update_status(&task).await?;
    Ok(())
}
