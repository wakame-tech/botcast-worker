pub mod episode_service;

use axum::async_trait;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub(crate) struct Episode {
    pub id: Uuid,
    pub title: String,
    pub audio_url: Option<String>,
    pub script_url: Option<String>,
    pub podcast_id: Uuid,
    pub user_id: Option<Uuid>,
    pub created_at: String,
}

#[async_trait]
pub(crate) trait EpisodeRepo: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Episode>>;
    async fn update(&self, episode: &Episode) -> anyhow::Result<()>;
}
