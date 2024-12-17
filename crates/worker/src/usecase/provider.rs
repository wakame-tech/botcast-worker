use super::{
    episode_service::EpisodeService, script_service::ScriptService, task_service::TaskService,
    ProvideApiClient, UserApiClientProvider,
};
use crate::r2_storage::ProvideStorage;
use repos::provider::*;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Provider {
    pub(crate) provide_podcast_repo: Arc<dyn ProvidePodcastRepo>,
    pub(crate) provide_episode_repo: Arc<dyn ProvideEpisodeRepo>,
    pub(crate) provide_task_repo: Arc<dyn ProvideTaskRepo>,
    pub(crate) provide_comment_repo: Arc<dyn ProvideCommentRepo>,
    pub(crate) provide_script_repo: Arc<dyn ProvideScriptRepo>,
    pub(crate) provide_storage: Arc<dyn ProvideStorage>,
    pub(crate) provide_secret_repo: Arc<dyn ProvideSecretRepo>,
    pub(crate) provide_api_client: Arc<dyn ProvideApiClient>,
}

impl Default for Provider {
    fn default() -> Self {
        Self {
            provide_podcast_repo: Arc::new(DefaultProvider),
            provide_episode_repo: Arc::new(DefaultProvider),
            provide_task_repo: Arc::new(DefaultProvider),
            provide_comment_repo: Arc::new(DefaultProvider),
            provide_script_repo: Arc::new(DefaultProvider),
            provide_storage: Arc::new(DefaultProvider),
            provide_secret_repo: Arc::new(DefaultProvider),
            provide_api_client: Arc::new(UserApiClientProvider::default()),
        }
    }
}

impl Provider {
    pub(crate) fn task_service(&self) -> TaskService {
        TaskService::new(
            self.provide_task_repo.task_repo(),
            self.provide_api_client.api_client(),
            self.episode_service(),
            self.script_service(),
        )
    }

    pub(crate) fn episode_service(&self) -> EpisodeService {
        EpisodeService::new(
            self.provide_episode_repo.episode_repo(),
            self.provide_storage.storage(),
        )
    }

    pub(crate) fn script_service(&self) -> ScriptService {
        ScriptService::new(
            self.provide_script_repo.script_repo(),
            self.provide_secret_repo.secret_repo(),
            self.provide_api_client.api_client(),
        )
    }
}
