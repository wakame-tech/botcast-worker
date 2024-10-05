use crate::task::{Task, TaskRepo, TaskStatus};
use axum::async_trait;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub(crate) struct PostgresTaskRepo {
    pub(crate) pool: Pool<Postgres>,
}

impl PostgresTaskRepo {
    pub(crate) fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
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
        sqlx::query("update tasks set status = $2, args = $3 where id = $1")
            .bind(task.id)
            .bind(&task.status as &TaskStatus)
            .bind(&task.args)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<()> {
        sqlx::query("delete from tasks where id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }
}
