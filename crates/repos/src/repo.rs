use crate::entity::{Comment, Episode, Script};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait EpisodeRepo: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<(Episode, Vec<Comment>)>>;
    async fn update(&self, episode: &Episode) -> anyhow::Result<()>;
}

#[async_trait]
pub trait CommentRepo: Send + Sync {
    async fn find_all(&self, episode_id: &Uuid) -> anyhow::Result<Vec<Comment>>;
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Comment>>;
}

#[async_trait]
pub trait ScriptRepo: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Script>>;
    async fn update(&self, script: &Script) -> anyhow::Result<()>;
}
