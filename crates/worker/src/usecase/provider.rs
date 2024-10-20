use repos::{
    postgres::*,
    provider::*,
    repo::{CommentRepo, EpisodeRepo, PodcastRepo, ScriptRepo, TaskRepo},
};
use std::{ops::Deref, sync::Arc};

#[derive(Debug, Clone, Copy)]
pub struct Provider<T = DefaultProvider>(T)
where
    T: ProvidePodcastRepo
        + ProvideEpisodeRepo
        + ProvideCommentRepo
        + ProvideScriptRepo
        + ProvideTaskRepo;

impl Default for Provider {
    fn default() -> Self {
        Self(DefaultProvider)
    }
}

impl Deref for Provider {
    type Target = DefaultProvider;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DefaultProvider;

impl ProvidePodcastRepo for DefaultProvider {
    fn podcast_repo(&self) -> Arc<dyn PodcastRepo> {
        Arc::new(PostgresPodcastRepo::new())
    }
}

impl ProvideEpisodeRepo for DefaultProvider {
    fn episode_repo(&self) -> Arc<dyn EpisodeRepo> {
        Arc::new(PostgresEpisodeRepo::new())
    }
}

impl ProvideCommentRepo for DefaultProvider {
    fn comment_repo(&self) -> Arc<dyn CommentRepo> {
        Arc::new(PostgresCommentRepo::new())
    }
}

impl ProvideScriptRepo for DefaultProvider {
    fn script_repo(&self) -> Arc<dyn ScriptRepo> {
        Arc::new(PostgresScriptRepo::new())
    }
}

impl ProvideTaskRepo for DefaultProvider {
    fn task_repo(&self) -> Arc<dyn TaskRepo> {
        Arc::new(PostgresTaskRepo::new())
    }
}
