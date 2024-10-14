pub mod entity;
pub mod postgres;
pub mod repo;

use crate::{
    postgres::{PostgresEpisodeRepo, PostgresScriptRepo},
    repo::{EpisodeRepo, ScriptRepo},
};
use std::sync::Arc;

pub fn episode_repo() -> Arc<dyn EpisodeRepo> {
    Arc::new(PostgresEpisodeRepo::new())
}

pub fn script_repo() -> Arc<dyn ScriptRepo> {
    Arc::new(PostgresScriptRepo::new())
}
