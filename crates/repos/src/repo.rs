use crate::entity::{Episode, Script};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait EpisodeRepo: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Episode>>;
    async fn update(&self, episode: &Episode) -> anyhow::Result<()>;
}

#[async_trait]
pub trait ScriptRepo: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Script>>;
    async fn update(&self, script: &Script) -> anyhow::Result<()>;
}
