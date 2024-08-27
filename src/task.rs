use crate::{scrape::ScrapeEpisode, synthesis::Synthesis, Ctx};
use std::fmt::Debug;
use surrealdb::{engine::local::Db, opt::RecordId, Surreal};
use uuid::Uuid;

pub(crate) trait RunTask
where
    Self: Debug + Clone + serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    fn id(&self) -> &RecordId;

    async fn run(&mut self, ctx: &Ctx) -> anyhow::Result<Option<Task>>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub(crate) enum Task {
    Scrape(ScrapeEpisode),
    Synthesis(Synthesis),
}

impl RunTask for Task {
    fn id(&self) -> &RecordId {
        match self {
            Self::Scrape(task) => &task.id,
            Self::Synthesis(task) => &task.id,
        }
    }

    async fn run(&mut self, ctx: &Ctx) -> anyhow::Result<Option<Task>> {
        match self {
            Self::Scrape(task) => task.run(ctx).await,
            Self::Synthesis(task) => task.run(ctx).await,
        }
    }
}

pub(crate) struct TaskRepo {
    pub(crate) db: Surreal<Db>,
}

impl TaskRepo {
    const TABLE: &'static str = "tasks";

    pub(crate) fn new_id() -> RecordId {
        RecordId::from((
            Self::TABLE.to_string(),
            Uuid::new_v4().as_hyphenated().to_string(),
        ))
    }

    pub(crate) async fn list(&self) -> anyhow::Result<Vec<Task>> {
        let tasks: Vec<Task> = self.db.select(Self::TABLE).await?;
        Ok(tasks)
    }

    pub(crate) async fn create(&self, task: Task) -> anyhow::Result<()> {
        self.db
            .create::<Option<Task>>(task.id())
            .content(task)
            .await?;
        Ok(())
    }

    pub(crate) async fn delete(&self, id: &RecordId) -> anyhow::Result<Task> {
        self.db
            .delete::<Option<Task>>(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))
    }

    pub(crate) async fn watch(&self, ctx: &Ctx) -> anyhow::Result<()> {
        let mut tasks = self.list().await?;
        if tasks.is_empty() {
            return Ok(());
        }
        log::info!("worker | {} tasks", tasks.len());
        for task in tasks.iter_mut() {
            match task.run(&ctx).await {
                Ok(Some(task)) => {
                    self.create(task).await?;
                }
                Err(e) => {
                    log::error!("Failed to run task: {:?}", e);
                }
                _ => {}
            };
            let res = self.delete(&task.id()).await?;
            log::info!("Deleted: {:?}", res.id().id.to_raw());
        }
        Ok(())
    }
}
