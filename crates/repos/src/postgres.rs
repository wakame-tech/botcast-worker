use crate::{
    entity::{
        Comment, CommentId, Episode, EpisodeId, Podcast, PodcastId, Script, ScriptId, Secret, Task,
        TaskId, TaskStatus,
    },
    error::Error,
    repo::{CommentRepo, EpisodeRepo, PodcastRepo, ScriptRepo, SecretRepo, TaskRepo},
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Pool, Postgres};
use std::sync::LazyLock;
use uuid::Uuid;

pub static PG_POOL: LazyLock<Pool<Postgres>> = LazyLock::new(|| {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
    PgPool::connect_lazy(&database_url).expect("Failed to connect to DB")
});

pub struct PostgresPodcastRepo {
    pool: Pool<Postgres>,
}

impl Default for PostgresPodcastRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl PostgresPodcastRepo {
    pub fn new() -> Self {
        let pool = PG_POOL.clone();
        Self { pool }
    }
}

#[async_trait]
impl PodcastRepo for PostgresPodcastRepo {
    async fn find_by_id(&self, id: &PodcastId) -> anyhow::Result<Podcast, Error> {
        let Some(podcast) = sqlx::query_as!(Podcast, "select * from podcasts where id = $1", id.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(Error::Other)?
        else {
            return Err(Error::NotFound("podcast".to_string(), id.0.to_string()));
        };
        Ok(podcast)
    }

    async fn update(&self, podcast: &Podcast) -> anyhow::Result<(), Error> {
        sqlx::query_as!(
            Podcast,
            "update podcasts set title = $2, icon = $3, script_id = $4, cron = $5 where id = $1",
            podcast.id,
            podcast.title,
            podcast.icon,
            podcast.script_id,
            podcast.cron,
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Other)?;
        Ok(())
    }
}

pub struct PostgresEpisodeRepo {
    pool: Pool<Postgres>,
}

impl Default for PostgresEpisodeRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl PostgresEpisodeRepo {
    pub fn new() -> Self {
        let pool = PG_POOL.clone();
        Self { pool }
    }
}

#[async_trait]
impl EpisodeRepo for PostgresEpisodeRepo {
    async fn find_by_id(&self, id: &EpisodeId) -> anyhow::Result<(Episode, Vec<Comment>), Error> {
        let Some(episode) = sqlx::query_as!(Episode, "select * from episodes where id = $1", id.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(Error::Other)?
        else {
            return Err(Error::NotFound("episode".to_string(), id.0.to_string()));
        };
        let comments = sqlx::query_as!(
            Comment,
            "select * from comments where episode_id = $1",
            id.0
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::Other)?;
        Ok((episode, comments))
    }

    async fn find_all_by_podcast_id(
        &self,
        podcast_id: &PodcastId,
    ) -> anyhow::Result<Vec<Episode>, Error> {
        let episodes = sqlx::query_as!(
            Episode,
            "select * from episodes where podcast_id = $1",
            podcast_id.0
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::Other)?;
        Ok(episodes)
    }

    async fn create(&self, episode: &Episode) -> anyhow::Result<(), Error> {
        sqlx::query_as!(
            Episode,
            "insert into episodes (id, user_id, title, podcast_id, sections, audio_url, srt_url) values ($1, $2, $3, $4, $5, $6, $7)",
            episode.id,
            episode.user_id,
            episode.title,
            episode.podcast_id,
            episode.sections,
            episode.audio_url,
            episode.srt_url,
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Other)?;
        Ok(())
    }

    async fn update(&self, episode: &Episode) -> anyhow::Result<(), Error> {
        sqlx::query_as!(
            Episode,
            "update episodes set title = $2, audio_url = $3, srt_url = $4 where id = $1",
            episode.id,
            episode.title,
            episode.audio_url,
            episode.srt_url,
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Other)?;
        Ok(())
    }
}

pub struct PostgresCommentRepo {
    pool: Pool<Postgres>,
}

impl Default for PostgresCommentRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl PostgresCommentRepo {
    pub fn new() -> Self {
        let pool = PG_POOL.clone();
        Self { pool }
    }
}

#[async_trait]
impl CommentRepo for PostgresCommentRepo {
    async fn find_all_by_episode_id(
        &self,
        episode_id: &EpisodeId,
    ) -> anyhow::Result<Vec<Comment>, Error> {
        let comments = sqlx::query_as!(
            Comment,
            "select * from comments where episode_id = $1",
            episode_id.0
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::Other)?;
        Ok(comments)
    }

    async fn find_by_id(&self, id: &CommentId) -> anyhow::Result<Comment, Error> {
        let Some(comment) = sqlx::query_as!(Comment, "select * from comments where id = $1", id.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(Error::Other)?
        else {
            return Err(Error::NotFound("comment".to_string(), id.0.to_string()));
        };
        Ok(comment)
    }
}

pub struct PostgresScriptRepo {
    pool: Pool<Postgres>,
}

impl Default for PostgresScriptRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl PostgresScriptRepo {
    pub fn new() -> Self {
        let pool = PG_POOL.clone();
        Self { pool }
    }
}

#[async_trait]
impl ScriptRepo for PostgresScriptRepo {
    async fn find_by_id(&self, id: &ScriptId) -> anyhow::Result<Script, Error> {
        let Some(script) = sqlx::query_as!(Script, "select * from scripts where id = $1", id.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(Error::Other)?
        else {
            return Err(Error::NotFound("script".to_string(), id.0.to_string()));
        };
        Ok(script)
    }

    async fn update(&self, script: &Script) -> anyhow::Result<(), Error> {
        sqlx::query_as!(
            Script,
            "update scripts set title = $2, template = $3 where id = $1",
            script.id,
            script.title,
            script.template,
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Other)?;
        Ok(())
    }
}

pub struct DummyScriptRepo {
    pub template: serde_json::Value,
}

#[async_trait]
impl ScriptRepo for DummyScriptRepo {
    async fn find_by_id(&self, id: &ScriptId) -> anyhow::Result<Script, Error> {
        let script = Script {
            id: id.0,
            user_id: Uuid::new_v4(),
            title: "dummy".to_string(),
            template: self.template.clone(),
        };
        Ok(script)
    }

    async fn update(&self, _script: &Script) -> anyhow::Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PostgresTaskRepo {
    pool: Pool<Postgres>,
}

impl Default for PostgresTaskRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl PostgresTaskRepo {
    pub fn new() -> Self {
        let pool = PG_POOL.clone();
        Self { pool }
    }
}

#[async_trait]
impl TaskRepo for PostgresTaskRepo {
    async fn pop(&self, now: DateTime<Utc>) -> anyhow::Result<Option<Task>, Error> {
        let task = sqlx::query_as!(
            Task,
            r#"select id, status as "status!: TaskStatus", args, result, execute_after, executed_at from tasks where status = $1 and execute_after < $2 order by execute_after limit 1"#,
            TaskStatus::Pending as TaskStatus,
            now,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::Other)?;
        Ok(task)
    }

    async fn create(&self, task: &Task) -> anyhow::Result<(), Error> {
        sqlx::query_as!(
            Task,
            "insert into tasks (id, status, args, result, execute_after, executed_at) values ($1, $2, $3, $4, $5, $6)",
            task.id,
            &task.status as &TaskStatus,
            task.args,
            task.result,
            task.execute_after,
            task.executed_at,
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Other)?;
        Ok(())
    }

    async fn update(&self, task: &Task) -> anyhow::Result<(), Error> {
        sqlx::query_as!(
            Task,
            "update tasks set status = $2, args = $3, result = $4, execute_after = $5, executed_at = $6 where id = $1",
            task.id,
            &task.status as &TaskStatus,
            task.args,
            task.result,
            task.execute_after,
            task.executed_at,
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Other)?;
        Ok(())
    }

    async fn delete(&self, id: &TaskId) -> anyhow::Result<(), Error> {
        sqlx::query("delete from tasks where id = $1")
            .bind(id.0)
            .fetch_one(&self.pool)
            .await
            .map_err(Error::Other)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PostgresSecretRepo {
    pool: Pool<Postgres>,
}

impl Default for PostgresSecretRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl PostgresSecretRepo {
    pub fn new() -> Self {
        let pool = PG_POOL.clone();
        Self { pool }
    }
}

#[async_trait]
impl SecretRepo for PostgresSecretRepo {
    async fn find_by_name(&self, user_id: &Uuid, name: &str) -> anyhow::Result<Secret, Error> {
        // NOTE: `name` is globally unique for users, so prefixed `user_uuid`
        let Some(secret) = sqlx::query_as!(
            Secret,
            "select name, decrypted_secret from vault.decrypted_secrets where name = $1",
            format!("{}:{}", user_id.hyphenated(), name),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::Other)?
        else {
            return Err(Error::NotFound("secret".to_string(), name.to_string()));
        };
        Ok(Secret {
            name: Some(name.to_string()),
            decrypted_secret: secret.decrypted_secret,
        })
    }
}
