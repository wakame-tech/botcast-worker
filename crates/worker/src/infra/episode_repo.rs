use crate::episode::{Episode, EpisodeRepo};
use axum::async_trait;
use chrono::Local;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

pub(crate) struct PostgresEpisodeRepo {
    pub(crate) pool: Pool<Postgres>,
}

impl PostgresEpisodeRepo {
    pub(crate) fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
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
            "update episodes set title = $2, audio_url = $3, manuscript = $4 where id = $1",
            episode.id,
            episode.title,
            episode.audio_url,
            episode.manuscript,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

pub(crate) struct DummyEpisodeRepo;

#[async_trait]
impl EpisodeRepo for DummyEpisodeRepo {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Episode>> {
        let episode = Episode {
            id: *id,
            title: "dummy".to_string(),
            audio_url: None,
            script_id: Uuid::new_v4(),
            manuscript: None,
            podcast_id: Uuid::new_v4(),
            user_id: None,
            created_at: Local::now().to_utc().to_rfc3339(),
        };
        Ok(Some(episode))
    }

    async fn update(&self, episode: &Episode) -> anyhow::Result<()> {
        log::info!("{:?}", episode);
        Ok(())
    }
}
