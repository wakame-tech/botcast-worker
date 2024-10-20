use crate::{
    entity::{
        Comment, CommentId, Episode, EpisodeId, Podcast, PodcastId, Script, ScriptId, Task, TaskId,
    },
    error::Error,
};
use async_trait::async_trait;

#[async_trait]
pub trait PodcastRepo: Send + Sync {
    async fn find_by_id(&self, id: &PodcastId) -> Result<Podcast, Error>;
    async fn update(&self, podcast: &Podcast) -> anyhow::Result<(), Error>;
}

#[async_trait]
pub trait EpisodeRepo: Send + Sync {
    async fn find_by_id(&self, id: &EpisodeId) -> anyhow::Result<(Episode, Vec<Comment>), Error>;
    async fn create(&self, episode: &Episode) -> anyhow::Result<(), Error>;
    async fn update(&self, episode: &Episode) -> anyhow::Result<(), Error>;
}

#[async_trait]
pub trait CommentRepo: Send + Sync {
    async fn find_all(&self, episode_id: &EpisodeId) -> anyhow::Result<Vec<Comment>, Error>;
    async fn find_by_id(&self, id: &CommentId) -> anyhow::Result<Option<Comment>, Error>;
}

#[async_trait]
pub trait ScriptRepo: Send + Sync {
    async fn find_by_id(&self, id: &ScriptId) -> anyhow::Result<Script, Error>;
    async fn update(&self, script: &Script) -> anyhow::Result<(), Error>;
}

#[async_trait]
pub trait TaskRepo: Send + Sync {
    async fn pop(&self) -> anyhow::Result<Option<Task>, Error>;
    async fn create(&self, task: &Task) -> anyhow::Result<(), Error>;
    async fn update(&self, task: &Task) -> anyhow::Result<(), Error>;
    #[allow(dead_code)]
    async fn delete(&self, id: &TaskId) -> anyhow::Result<(), Error>;
}
