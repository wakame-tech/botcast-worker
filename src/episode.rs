use sqlx::{Pool, Postgres};

#[derive(Debug, serde::Serialize)]
pub(crate) struct Episode {
    pub id: String,
    pub title: String,
    pub content: String,
    pub audio_url: Option<String>,
}

pub(crate) struct EpisodeRepo {
    pub(crate) pool: Pool<Postgres>,
}

impl EpisodeRepo {
    pub(crate) async fn list(&self) -> anyhow::Result<Vec<Episode>> {
        let episodes = sqlx::query_as!(
            Episode,
            r#"
            SELECT id, title, content, audio_url
            FROM episodes
            ORDER BY id DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(episodes)
    }
}
