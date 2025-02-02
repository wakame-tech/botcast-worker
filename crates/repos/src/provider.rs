use crate::{postgres::*, repo::*};
use std::{fmt::Debug, sync::Arc};

pub trait ProvidePodcastRepo: Debug + Send + Sync {
    fn podcast_repo(&self) -> Arc<dyn PodcastRepo>;
}

pub trait ProvideEpisodeRepo: Debug + Send + Sync {
    fn episode_repo(&self) -> Arc<dyn EpisodeRepo>;
}

pub trait ProvideScriptRepo: Debug + Send + Sync {
    fn script_repo(&self) -> Arc<dyn ScriptRepo>;
}

pub trait ProvideCornerRepo: Debug + Send + Sync {
    fn corner_repo(&self) -> Arc<dyn CornerRepo>;
}

pub trait ProvideMailRepo: Debug + Send + Sync {
    fn mail_repo(&self) -> Arc<dyn MailRepo>;
}

pub trait ProvideTaskRepo: Debug + Send + Sync {
    fn task_repo(&self) -> Arc<dyn TaskRepo>;
}

pub trait ProvideSecretRepo: Debug + Send + Sync {
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

impl ProvideScriptRepo for DefaultProvider {
    fn script_repo(&self) -> Arc<dyn ScriptRepo> {
        Arc::new(PostgresScriptRepo::new())
    }
}

impl ProvideCornerRepo for DefaultProvider {
    fn corner_repo(&self) -> Arc<dyn CornerRepo> {
        Arc::new(PostgresCornerRepo::new())
    }
}

impl ProvideMailRepo for DefaultProvider {
    fn mail_repo(&self) -> Arc<dyn MailRepo> {
        Arc::new(PostgresMailRepo::new())
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
