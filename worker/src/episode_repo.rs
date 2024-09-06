use crate::episode::Episode;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

pub(crate) trait EpisodeRepo {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Episode>>;
    async fn update(&mut self, episode: &Episode) -> anyhow::Result<()>;
}

pub(crate) struct PostgresEpisodeRepo {
    pub(crate) pool: Pool<Postgres>,
}

impl PostgresEpisodeRepo {
    pub(crate) fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

impl EpisodeRepo for PostgresEpisodeRepo {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Episode>> {
        let episode = sqlx::query_as!(Episode, "select * from episodes where id = $1", id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(episode)
    }

    async fn update(&mut self, episode: &Episode) -> anyhow::Result<()> {
        sqlx::query_as!(
            Episode,
            "update episodes set title = $2, content = $3, audio_url = $4 where id = $1",
            episode.id,
            episode.title,
            episode.content,
            episode.audio_url
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(())
    }
}

pub(crate) struct InMemoryEpisodeRepo {
    episodes: std::collections::HashMap<Uuid, Episode>,
}

impl InMemoryEpisodeRepo {
    pub(crate) fn new() -> Self {
        Self {
            episodes: std::collections::HashMap::new(),
        }
    }
}

impl EpisodeRepo for InMemoryEpisodeRepo {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Episode>> {
        Ok(self.episodes.get(id).cloned())
    }

    async fn update(&mut self, episode: &Episode) -> anyhow::Result<()> {
        self.episodes.insert(episode.id, episode.clone());
        Ok(())
    }
}
