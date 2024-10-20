use crate::{
    postgres::{
        PostgresCommentRepo, PostgresEpisodeRepo, PostgresPodcastRepo, PostgresScriptRepo,
        PostgresTaskRepo,
    },
    repo::{CommentRepo, EpisodeRepo, PodcastRepo, ScriptRepo, TaskRepo},
};
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub struct Provider;

pub trait ProvidePodcastRepo {
    fn podcast_repo(&self) -> Arc<dyn PodcastRepo>;
}

impl ProvidePodcastRepo for Provider {
    fn podcast_repo(&self) -> Arc<dyn PodcastRepo> {
        Arc::new(PostgresPodcastRepo::new())
    }
}

pub trait ProvideEpisodeRepo {
    fn episode_repo(&self) -> Arc<dyn EpisodeRepo>;
}

impl ProvideEpisodeRepo for Provider {
    fn episode_repo(&self) -> Arc<dyn EpisodeRepo> {
        Arc::new(PostgresEpisodeRepo::new())
    }
}

pub trait ProvideCommentRepo {
    fn comment_repo(&self) -> Arc<dyn CommentRepo>;
}

impl ProvideCommentRepo for Provider {
    fn comment_repo(&self) -> Arc<dyn CommentRepo> {
        Arc::new(PostgresCommentRepo::new())
    }
}

pub trait ProvideScriptRepo {
    fn script_repo(&self) -> Arc<dyn ScriptRepo>;
}

impl ProvideScriptRepo for Provider {
    fn script_repo(&self) -> Arc<dyn ScriptRepo> {
        Arc::new(PostgresScriptRepo::new())
    }
}

pub trait ProvideTaskRepo {
    fn task_repo(&self) -> Arc<dyn TaskRepo>;
}

impl ProvideTaskRepo for Provider {
    fn task_repo(&self) -> Arc<dyn TaskRepo> {
        Arc::new(PostgresTaskRepo::new())
    }
}
