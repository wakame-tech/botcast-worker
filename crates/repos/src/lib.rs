pub mod entity;
pub mod error;
pub mod postgres;
pub mod repo;
pub mod urn;

use crate::{
    postgres::{PostgresEpisodeRepo, PostgresScriptRepo},
    repo::{EpisodeRepo, ScriptRepo},
};
use postgres::{PostgresCommentRepo, PostgresPodcastRepo, PostgresTaskRepo};
use repo::{CommentRepo, PodcastRepo, TaskRepo};
use std::sync::Arc;

pub fn podcast_repo() -> Arc<dyn PodcastRepo> {
    Arc::new(PostgresPodcastRepo::new())
}

pub fn episode_repo() -> Arc<dyn EpisodeRepo> {
    Arc::new(PostgresEpisodeRepo::new())
}

pub fn comment_repo() -> Arc<dyn CommentRepo> {
    Arc::new(PostgresCommentRepo::new())
}

pub fn script_repo() -> Arc<dyn ScriptRepo> {
    Arc::new(PostgresScriptRepo::new())
}

pub fn task_repo() -> Arc<dyn TaskRepo> {
    Arc::new(PostgresTaskRepo::new())
}
