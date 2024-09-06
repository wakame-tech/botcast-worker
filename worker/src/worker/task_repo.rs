use super::task::Task;
use crate::worker::task::TaskStatus;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[allow(dead_code)]
pub(crate) trait TaskRepo {
    async fn pop(&self) -> anyhow::Result<Option<Task>>;
    async fn create(&self, task: &Task) -> anyhow::Result<()>;
    async fn update(&self, task: &Task) -> anyhow::Result<()>;
    async fn delete(&self, id: &Uuid) -> anyhow::Result<Task>;
}

pub(crate) struct PostgresTaskRepo {
    pub(crate) pool: Pool<Postgres>,
}

impl PostgresTaskRepo {
    pub(crate) fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

impl TaskRepo for PostgresTaskRepo {
    async fn pop(&self) -> anyhow::Result<Option<Task>> {
        let task =
            sqlx::query_as(r#"select * from tasks where status = 'PENDING' order by id limit 1"#)
                .fetch_optional(&self.pool)
                .await?;
        Ok(task)
    }

    async fn create(&self, task: &Task) -> anyhow::Result<()> {
        sqlx::query!(
            "insert into tasks (id, status, args) values ($1, $2, $3)",
            task.id,
            &task.status as &TaskStatus,
            task.args,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, task: &Task) -> anyhow::Result<()> {
        sqlx::query_as("update tasks set status = $2, args = $3 where id = $1 returning *")
            .bind(task.id)
            .bind(&task.status as &TaskStatus)
            .bind(&task.args)
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<Task> {
        let task = sqlx::query_as("delete from tasks where id = $1 returning *")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(task)
    }
}
