use super::episode::Episode;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

pub(crate) trait EpisodeRepo {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Episode>>;
    async fn update(&self, episode: &Episode) -> anyhow::Result<()>;
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

    async fn update(&self, episode: &Episode) -> anyhow::Result<()> {
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

pub(crate) struct DummyEpisodeRepo;

impl EpisodeRepo for DummyEpisodeRepo {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Episode>> {
        let episode = Episode {
            id: *id,
            title: "dummy".to_string(),
            content: None,
            audio_url: None,
        };
        Ok(Some(episode))
    }

    async fn update(&self, episode: &Episode) -> anyhow::Result<()> {
        log::info!("{}", episode);
        Ok(())
    }
}
