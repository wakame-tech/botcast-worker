pub mod episode_service;
pub mod script_service;

use axum::async_trait;
use uuid::Uuid;

use crate::infra::workdir::WorkDir;

fn use_work_dir(task_id: &Uuid) -> anyhow::Result<WorkDir> {
    let keep = std::env::var("KEEP_WORKDIR")
        .unwrap_or("false".to_string())
        .parse()?;
    WorkDir::new(task_id, keep)
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub(crate) struct Episode {
    pub id: Uuid,
    pub title: String,
    pub audio_url: Option<String>,
    pub script_id: Uuid,
    pub manuscript: Option<serde_json::Value>,
    pub podcast_id: Uuid,
    pub user_id: Option<Uuid>,
    pub created_at: String,
}

#[async_trait]
pub(crate) trait EpisodeRepo: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Episode>>;
    async fn update(&self, episode: &Episode) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub(crate) struct Script {
    pub id: Uuid,
    pub user_id: Uuid,
    pub template: serde_json::Value,
}

#[async_trait]
pub(crate) trait ScriptRepo: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Script>>;
    async fn update(&self, script: &Script) -> anyhow::Result<()>;
}
