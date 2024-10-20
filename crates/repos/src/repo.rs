use crate::entity::{
    Comment, CommentId, Episode, EpisodeId, Podcast, PodcastId, Script, ScriptId, Task, TaskId,
};
use async_trait::async_trait;

#[async_trait]
pub trait PodcastRepo: Send + Sync {
    async fn find_by_id(&self, id: &PodcastId) -> anyhow::Result<Podcast>;
    async fn update(&self, podcast: &Podcast) -> anyhow::Result<()>;
}

#[async_trait]
pub trait EpisodeRepo: Send + Sync {
    async fn find_by_id(&self, id: &EpisodeId) -> anyhow::Result<(Episode, Vec<Comment>)>;
    async fn create(&self, episode: &Episode) -> anyhow::Result<()>;
    async fn update(&self, episode: &Episode) -> anyhow::Result<()>;
}

#[async_trait]
pub trait CommentRepo: Send + Sync {
    async fn find_all(&self, episode_id: &EpisodeId) -> anyhow::Result<Vec<Comment>>;
    async fn find_by_id(&self, id: &CommentId) -> anyhow::Result<Option<Comment>>;
}

#[async_trait]
pub trait ScriptRepo: Send + Sync {
    async fn find_by_id(&self, id: &ScriptId) -> anyhow::Result<Script>;
    async fn update(&self, script: &Script) -> anyhow::Result<()>;
}

#[async_trait]
pub trait TaskRepo: Send + Sync {
    async fn pop(&self) -> anyhow::Result<Option<Task>>;
    async fn create(&self, task: &Task) -> anyhow::Result<()>;
    async fn update(&self, task: &Task) -> anyhow::Result<()>;
    #[allow(dead_code)]
    async fn delete(&self, id: &TaskId) -> anyhow::Result<()>;
}
