use crate::{
    episode::episode_service::EpisodeService,
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
        let episode_repo = PostgresEpisodeRepo::new(pool.clone());
        let storage = R2Storage::new().expect("Failed to create storage");
        let episode_service = Arc::new(EpisodeService {
            episode_repo: Box::new(episode_repo),
            storage: Box::new(storage),
            synthesizer: Box::new(VoiceVoxAudioSynthesizer::default()),
        });

        let task_repo = PostgresTaskRepo::new(pool.clone());
        let task_service = Arc::new(TaskService {
            task_repo: Box::new(task_repo),
            episode_service: episode_service.clone(),
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
