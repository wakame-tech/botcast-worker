pub mod episode_repo;
pub mod http_client;
pub mod r2_storage;
pub mod task_repo;
pub mod voicevox_client;
pub mod workdir;

use axum::async_trait;

// TODO: place appropriate modules
#[async_trait]
pub(crate) trait Storage: Send + Sync {
    async fn upload(&self, path: &str, data: &[u8], content_type: &str) -> anyhow::Result<()>;
    fn get_endpoint(&self) -> String;
}
