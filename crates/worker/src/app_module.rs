use crate::{
    episode::{episode_service::EpisodeService, script_service::ScriptService},
    infra::{r2_storage::R2Storage, task_repo::PostgresTaskRepo},
    task::task_service::TaskService,
};
use repos::postgres::{PostgresEpisodeRepo, PostgresScriptRepo};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppModule {
    pub(crate) task_service: TaskService,
    pub(crate) episode_service: EpisodeService,
    pub(crate) script_service: ScriptService,
}

impl AppModule {
    pub async fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to DB");

        let episode_repo = Arc::new(PostgresEpisodeRepo::new(pool.clone()));
        let storage = Arc::new(R2Storage::new().expect("Failed to create storage"));
        let script_repo = Arc::new(PostgresScriptRepo::new(pool.clone()));
        let script_service = ScriptService {
            script_repo: script_repo.clone(),
        };
        let episode_service = EpisodeService {
            script_repo: script_repo.clone(),
            script_service: script_service.clone(),
            episode_repo: episode_repo.clone(),
            storage: storage.clone(),
        };

        let task_repo = Arc::new(PostgresTaskRepo::new(pool.clone()));
        let task_service = TaskService {
            task_repo: task_repo.clone(),
            episode_service: episode_service.clone(),
            script_service: script_service.clone(),
        };

        Self {
            task_service,
            episode_service,
            script_service,
        }
    }
}
