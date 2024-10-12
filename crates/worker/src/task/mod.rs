pub mod postgres;
pub mod task_service;

use axum::async_trait;
use postgres::PostgresTaskRepo;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, sqlx::Type, serde::Serialize, serde::Deserialize)]
#[sqlx(type_name = "task_status", rename_all = "UPPERCASE")]
pub(crate) enum TaskStatus {
    None,
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub(crate) struct Task {
    pub id: Uuid,
    pub status: TaskStatus,
    pub args: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Args {
    pub(crate) episode_id: Uuid,
    pub(crate) url: String,
}

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
