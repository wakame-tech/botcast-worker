use crate::{
    postgres::*,
    repo::{CommentRepo, EpisodeRepo, PodcastRepo, ScriptRepo, SecretRepo, TaskRepo},
};
use std::sync::Arc;

pub trait ProvidePodcastRepo {
    fn podcast_repo(&self) -> Arc<dyn PodcastRepo>;
}

pub trait ProvideEpisodeRepo {
    fn episode_repo(&self) -> Arc<dyn EpisodeRepo>;
}

pub trait ProvideCommentRepo {
    fn comment_repo(&self) -> Arc<dyn CommentRepo>;
}

pub trait ProvideScriptRepo {
    fn script_repo(&self) -> Arc<dyn ScriptRepo>;
}

pub trait ProvideTaskRepo {
    fn task_repo(&self) -> Arc<dyn TaskRepo>;
}

pub trait ProvideSecretRepo {
    fn secret_repo(&self) -> Arc<dyn SecretRepo>;
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

impl ProvideSecretRepo for DefaultProvider {
    fn secret_repo(&self) -> Arc<dyn SecretRepo> {
        Arc::new(PostgresSecretRepo::new())
    }
}
