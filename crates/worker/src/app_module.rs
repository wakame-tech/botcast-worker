use crate::{
    episode::{episode_service::EpisodeService, script_service::ScriptService},
    infra::{
        episode_repo::PostgresEpisodeRepo, r2_storage::R2Storage, task_repo::PostgresTaskRepo,
        voicevox_synthesizer::VoiceVoxAudioSynthesizer,
    },
    task::task_service::TaskService,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppModule {
    pub(crate) task_service: TaskService,
    pub(crate) episode_service: EpisodeService,
    pub(crate) scrape_service: ScriptService,
}

impl AppModule {
    pub async fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to DB");

        let episode_repo = Arc::new(PostgresEpisodeRepo::new(pool.clone()));
        let storage = Arc::new(R2Storage::new().expect("Failed to create storage"));
        let synthesizer = Arc::new(VoiceVoxAudioSynthesizer::default());
        let episode_service = EpisodeService {
            episode_repo: episode_repo.clone(),
            storage: storage.clone(),
            synthesizer: synthesizer.clone(),
        };
        let scrape_service = ScriptService {
            episode_repo: episode_repo.clone(),
        };

        let task_repo = Arc::new(PostgresTaskRepo::new(pool.clone()));
        let task_service = TaskService {
            task_repo: task_repo.clone(),
            episode_service: episode_service.clone(),
            scrape_service: scrape_service.clone(),
        };

        Self {
            task_service,
            episode_service,
            scrape_service,
        }
    }
}
