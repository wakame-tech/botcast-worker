use super::{
    episode_service::EpisodeService, script_service::ScriptService, task_service::TaskService,
};
use crate::r2_storage::ProvideStorage;
use repos::provider::*;

#[derive(Debug, Clone, Copy)]
pub struct Provider<
    ProvidePodcastRepo = DefaultProvider,
    ProvideEpisodeRepo = DefaultProvider,
    ProvideTaskRepo = DefaultProvider,
    ProvideCommentRepo = DefaultProvider,
    ProvideScriptRepo = DefaultProvider,
    ProvideStorage = DefaultProvider,
> {
    pub(crate) provide_podcast_repo: ProvidePodcastRepo,
    pub(crate) provide_episode_repo: ProvideEpisodeRepo,
    pub(crate) provide_task_repo: ProvideTaskRepo,
    pub(crate) provide_comment_repo: ProvideCommentRepo,
    pub(crate) provide_script_repo: ProvideScriptRepo,
    pub(crate) provide_storage: ProvideStorage,
}

impl Default for Provider {
    fn default() -> Self {
        Self {
            provide_podcast_repo: DefaultProvider,
            provide_episode_repo: DefaultProvider,
            provide_task_repo: DefaultProvider,
            provide_comment_repo: DefaultProvider,
            provide_script_repo: DefaultProvider,
            provide_storage: DefaultProvider,
        }
    }
}

impl Provider {
    pub(crate) fn task_service(&self) -> TaskService {
        TaskService::new(
            self.provide_task_repo.task_repo(),
            self.episode_service(),
            self.script_service(),
        )
    }

    pub(crate) fn episode_service(&self) -> EpisodeService {
        EpisodeService::new(
            self.provide_podcast_repo.podcast_repo(),
            self.provide_episode_repo.episode_repo(),
            self.provide_script_repo.script_repo(),
            self.provide_storage.storage(),
            self.script_service(),
        )
    }

    pub(crate) fn script_service(&self) -> ScriptService {
        ScriptService::new(
            self.provide_podcast_repo.podcast_repo(),
            self.provide_episode_repo.episode_repo(),
            self.provide_comment_repo.comment_repo(),
            self.provide_script_repo.script_repo(),
        )
    }
}
