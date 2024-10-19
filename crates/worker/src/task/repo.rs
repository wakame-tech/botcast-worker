use crate::task::model::{Task, TaskStatus};
use axum::async_trait;
use chrono::Utc;
use repos::postgres::PG_POOL;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use uuid::Uuid;

#[async_trait]
pub(crate) trait TaskRepo: Send + Sync {
    async fn pop(&self) -> anyhow::Result<Option<Task>>;
    async fn create(&self, task: &Task) -> anyhow::Result<()>;
    async fn update(&self, task: &Task) -> anyhow::Result<()>;
    async fn delete(&self, id: &Uuid) -> anyhow::Result<()>;
}

pub(crate) fn task_repo() -> Arc<dyn TaskRepo> {
    Arc::new(PostgresTaskRepo::new())
}

#[derive(Debug, Clone)]
pub(crate) struct PostgresTaskRepo {
    pub(crate) pool: Pool<Postgres>,
}

impl PostgresTaskRepo {
    pub(crate) fn new() -> Self {
        let pool = PG_POOL.clone();
        Self { pool }
    }
}

#[async_trait]
impl TaskRepo for PostgresTaskRepo {
    async fn pop(&self) -> anyhow::Result<Option<Task>> {
        let task = sqlx::query_as!(
            Task,
            r#"select id, status as "status!: TaskStatus", args, execute_after, executed_at from tasks where status = $1 and $2 < execute_after order by id limit 1"#,
            TaskStatus::Pending as TaskStatus,
            Utc::now(),
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(task)
    }

    async fn create(&self, task: &Task) -> anyhow::Result<()> {
        sqlx::query_as!(
            Task,
            "insert into tasks (id, status, args, execute_after, executed_at) values ($1, $2, $3, $4, $5)",
            task.id,
            &task.status as &TaskStatus,
            task.args,
            task.execute_after,
            task.executed_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, task: &Task) -> anyhow::Result<()> {
        sqlx::query_as!(
            Task,
            "update tasks set status = $2, args = $3, execute_after = $4, executed_at = $5 where id = $1",
            task.id,
            &task.status as &TaskStatus,
            task.args,
            task.execute_after,
            task.executed_at,
        )
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
