use repos::{
    postgres::{PostgresCommentRepo, PostgresEpisodeRepo, PostgresPodcastRepo, PostgresScriptRepo},
    provider::{ProvideCommentRepo, ProvideEpisodeRepo, ProvidePodcastRepo, ProvideScriptRepo},
    repo::{CommentRepo, EpisodeRepo, PodcastRepo, ScriptRepo},
};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct DefaultProvider;

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
