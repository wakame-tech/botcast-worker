pub mod api;
mod speaker;

use crate::AudioGenerator;
use anyhow::Result;
use api::VoiceVoxClient;
use async_trait::async_trait;

#[async_trait]
impl AudioGenerator for VoiceVoxClient {
    async fn generate(&self, speaker_id: &str, text: &str) -> Result<Vec<u8>> {
        let speaker = serde_json::from_str(speaker_id)?;
        let query = self.query(text, &speaker).await?;
        let audio = self.synthesis(query, &speaker).await?;
        Ok(audio)
    }
}
