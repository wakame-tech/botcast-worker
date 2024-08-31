use crate::model::{Episode, Task, TaskStatus};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

pub(crate) struct EpisodeRepo {
    pub(crate) pool: Pool<Postgres>,
}

impl EpisodeRepo {
    pub(crate) fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub(crate) async fn list(&self) -> anyhow::Result<Vec<Episode>> {
        let episodes = sqlx::query_as!(Episode, "select * from episodes order by id desc",)
            .fetch_all(&self.pool)
            .await?;
        Ok(episodes)
    }
}

pub(crate) struct TaskRepo {
    pub(crate) pool: Pool<Postgres>,
}

impl TaskRepo {
    pub(crate) fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub(crate) async fn list(&self) -> anyhow::Result<Vec<Task>> {
        let tasks: Vec<Task> = sqlx::query_as(r#"select * from tasks order by id desc"#)
            .fetch_all(&self.pool)
            .await?;
        Ok(tasks)
    }

    pub(crate) async fn create(&self, task: Task) -> anyhow::Result<()> {
        sqlx::query!(
            "insert into tasks (id, status, args) values ($1, $2, $3)",
            task.id,
            task.status as TaskStatus,
            task.args,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub(crate) async fn delete(&self, id: &Uuid) -> anyhow::Result<Task> {
        let task = sqlx::query_as("delete from tasks where id = $1 returning *")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(task)
    }
}
