use crate::{
    episode::{episode_service::EpisodeService, scrape_service::ScrapeService},
    infra::{
        episode_repo::PostgresEpisodeRepo, r2_storage::R2Storage, task_repo::PostgresTaskRepo,
        voicevox_synthesizer::VoiceVoxAudioSynthesizer,
    },
    task::task_service::TaskService,
};
use std::{sync::Arc, time::Duration};

pub fn start_worker() {
    tokio::spawn(async move {
        let interval = Duration::from_secs(5);
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to DB");
        let episode_repo = Arc::new(PostgresEpisodeRepo::new(pool.clone()));
        let storage = Arc::new(R2Storage::new().expect("Failed to create storage"));
        let synthesizer = Arc::new(VoiceVoxAudioSynthesizer::default());
        let episode_service = Arc::new(EpisodeService {
            episode_repo: episode_repo.clone(),
            storage: storage.clone(),
            synthesizer: synthesizer.clone(),
        });
        let scrape_service = Arc::new(ScrapeService {
            episode_repo: episode_repo.clone(),
        });

        let task_repo = Arc::new(PostgresTaskRepo::new(pool.clone()));
        let task_service = Arc::new(TaskService {
            task_repo: task_repo.clone(),
            episode_service: episode_service.clone(),
            scrape_service,
        });

        loop {
            log::info!("Watching tasks...");
            if let Err(e) = task_service.batch().await {
                log::error!("Error: {:?}", e);
            }
            tokio::time::sleep(interval).await;
        }
    });
}
