pub mod voicevox;

use async_trait::async_trait;

#[async_trait]
pub trait AudioGenerator {
    async fn generate(&self, speaker_id: &str, text: &str) -> anyhow::Result<Vec<u8>>;
}
