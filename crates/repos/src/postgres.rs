use crate::{
    entity::{Comment, Episode, Script},
    repo::{CommentRepo, EpisodeRepo, ScriptRepo},
};
use async_trait::async_trait;
use chrono::Local;
use sqlx::{PgPool, Pool, Postgres};
use std::sync::LazyLock;
use uuid::Uuid;

pub struct PostgresEpisodeRepo {
    pool: Pool<Postgres>,
}

pub static PG_POOL: LazyLock<Pool<Postgres>> = LazyLock::new(|| {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
    PgPool::connect_lazy(&database_url).expect("Failed to connect to DB")
});

impl PostgresEpisodeRepo {
    pub fn new() -> Self {
        let pool = PG_POOL.clone();
        Self { pool }
    }
}

#[async_trait]
impl EpisodeRepo for PostgresEpisodeRepo {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<(Episode, Vec<Comment>)>> {
        let Some(episode) = sqlx::query_as!(Episode, "select * from episodes where id = $1", id)
            .fetch_optional(&self.pool)
            .await?
        else {
            return Ok(None);
        };
        let comments = sqlx::query_as!(Comment, "select * from comments where episode_id = $1", id)
            .fetch_all(&self.pool)
            .await?;
        Ok(Some((episode, comments)))
    }

    async fn update(&self, episode: &Episode) -> anyhow::Result<()> {
        sqlx::query_as!(
            Episode,
            "update episodes set title = $2, audio_url = $3, srt_url = $4 where id = $1",
            episode.id,
            episode.title,
            episode.audio_url,
            episode.srt_url,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

pub struct DummyEpisodeRepo;

#[async_trait]
impl EpisodeRepo for DummyEpisodeRepo {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<(Episode, Vec<Comment>)>> {
        let episode = Episode {
            id: *id,
            title: "dummy".to_string(),
            audio_url: None,
            script_id: Uuid::new_v4(),
            srt_url: None,
            podcast_id: Uuid::new_v4(),
            user_id: None,
            created_at: Local::now().to_utc().to_rfc3339(),
        };
        Ok(Some((episode, vec![])))
    }

    async fn update(&self, episode: &Episode) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct PostgresCommentRepo {
    pool: Pool<Postgres>,
}

impl PostgresCommentRepo {
    pub fn new() -> Self {
        let pool = PG_POOL.clone();
        Self { pool }
    }
}

#[async_trait]
impl CommentRepo for PostgresCommentRepo {
    async fn find_all(&self, episode_id: &Uuid) -> anyhow::Result<Vec<Comment>> {
        let comments = sqlx::query_as!(
            Comment,
            "select * from comments where episode_id = $1",
            episode_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(comments)
    }

    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Comment>> {
        let comment = sqlx::query_as!(Comment, "select * from comments where id = $1", id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(comment)
    }
}

pub struct PostgresScriptRepo {
    pool: Pool<Postgres>,
}

impl PostgresScriptRepo {
    pub fn new() -> Self {
        let pool = PG_POOL.clone();
        Self { pool }
    }
}

#[async_trait]
impl ScriptRepo for PostgresScriptRepo {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Script>> {
        let script = sqlx::query_as!(Script, "select * from scripts where id = $1", id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(script)
    }

    async fn update(&self, script: &Script) -> anyhow::Result<()> {
        sqlx::query_as!(
            Episode,
            "update scripts set title = $2, template = $3, result = $4 where id = $1",
            script.id,
            script.title,
            script.template,
            script.result,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

pub struct DummyScriptRepo {
    pub template: serde_json::Value,
}

#[async_trait]
impl ScriptRepo for DummyScriptRepo {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Script>> {
        let script = Script {
            id: *id,
            user_id: Uuid::new_v4(),
            title: "dummy".to_string(),
            template: self.template.clone(),
            result: None,
        };
        Ok(Some(script))
    }

    async fn update(&self, script: &Script) -> anyhow::Result<()> {
        Ok(())
    }
}
