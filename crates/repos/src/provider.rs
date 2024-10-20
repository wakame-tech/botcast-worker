use crate::repo::{CommentRepo, EpisodeRepo, PodcastRepo, ScriptRepo, TaskRepo};
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
