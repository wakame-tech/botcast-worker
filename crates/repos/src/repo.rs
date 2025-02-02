use crate::{
    entity::{
        Corner, CornerId, Episode, EpisodeId, Podcast, PodcastId, Script, ScriptId, Secret, Task,
        TaskId,
    },
    error::Error,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait]
pub trait PodcastRepo: Send + Sync {
    async fn find_by_id(&self, id: &PodcastId) -> Result<Podcast, Error>;
    async fn update(&self, podcast: &Podcast) -> anyhow::Result<(), Error>;
}

#[async_trait]
pub trait EpisodeRepo: Send + Sync {
    async fn find_by_id(&self, id: &EpisodeId) -> anyhow::Result<Episode, Error>;
    async fn find_all_by_podcast_id(
        &self,
        podcast_id: &PodcastId,
    ) -> anyhow::Result<Vec<Episode>, Error>;
    async fn create(&self, episode: &Episode) -> anyhow::Result<(), Error>;
    async fn update(&self, episode: &Episode) -> anyhow::Result<(), Error>;
}

#[async_trait]
pub trait ScriptRepo: Send + Sync {
    async fn find_by_id(&self, id: &ScriptId) -> anyhow::Result<Script, Error>;
    async fn update(&self, script: &Script) -> anyhow::Result<(), Error>;
}

#[async_trait]
pub trait CornerRepo: Send + Sync {
    async fn find_by_id(&self, id: &CornerId) -> anyhow::Result<Corner, Error>;
    async fn update(&self, script: &Corner) -> anyhow::Result<(), Error>;
}

#[async_trait]
pub trait TaskRepo: Send + Sync {
    async fn pop(&self, now: DateTime<Utc>) -> anyhow::Result<Option<Task>, Error>;
    async fn create(&self, task: &Task) -> anyhow::Result<(), Error>;
    async fn update(&self, task: &Task) -> anyhow::Result<(), Error>;
    #[allow(dead_code)]
    async fn delete(&self, id: &TaskId) -> anyhow::Result<(), Error>;
}

#[async_trait]
pub trait SecretRepo: Send + Sync {
    async fn find_by_name(&self, user_id: &Uuid, name: &str) -> anyhow::Result<Secret, Error>;
}
