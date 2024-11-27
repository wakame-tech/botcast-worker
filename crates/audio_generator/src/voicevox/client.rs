use super::speaker::VoiceVoxSpeaker;
use crate::AudioGenerator;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use tracing::instrument;

#[async_trait]
impl AudioGenerator for VoiceVoxClient {
    #[instrument(skip(self))]
    async fn generate(&self, speaker_id: &str, text: &str) -> Result<Vec<u8>> {
        let speaker = speaker_id.parse()?;
        let query = self.query(text, &speaker).await?;
        let audio = self.synthesis(query, &speaker).await?;
        Ok(audio)
    }
}

#[derive(Debug, Clone)]
pub struct VoiceVoxClient {
    endpoint: String,
    client: reqwest::Client,
}

impl VoiceVoxClient {
    pub fn new(endpoint: String) -> Self {
        tracing::info!("VoiceVox endpoint: {}", endpoint);
        Self {
            endpoint,
            client: reqwest::Client::new(),
        }
    }

    pub(crate) async fn version(&self) -> anyhow::Result<Value> {
        let url = format!("{}/version", self.endpoint);
        let res = self.client.get(url).send().await?;
        if res.status() != reqwest::StatusCode::OK {
            anyhow::bail!(
                "Failed to get version: {} {}",
                res.status(),
                res.json::<Value>().await?.to_string()
            );
        }
        let res = res.json().await?;
        Ok(res)
    }

    pub(crate) async fn query(
        &self,
        text: &str,
        speaker: &VoiceVoxSpeaker,
    ) -> anyhow::Result<Value> {
        let url = format!(
            "{}/audio_query?text={}&speaker={}",
            self.endpoint,
            urlencoding::encode(text),
            speaker.id()
        );
        tracing::info!("Query: {}", url);
        let res = self.client.post(url).send().await?;
        if res.status() != reqwest::StatusCode::OK {
            anyhow::bail!(
                "Failed to query: {} {}",
                res.status(),
                res.json::<Value>().await?.to_string()
            );
        }
        let res = res.json().await?;
        Ok(res)
    }

    pub(crate) async fn synthesis(
        &self,
        query: Value,
        speaker: &VoiceVoxSpeaker,
    ) -> anyhow::Result<Vec<u8>> {
        let url = format!("{}/synthesis?speaker={}", self.endpoint, speaker.id());
        tracing::info!("Synthesis: {}", url);
        let res = self.client.post(url).json(&query).send().await?;
        if res.status() != reqwest::StatusCode::OK {
            anyhow::bail!(
                "Failed to synthesis: {} {}",
                res.status(),
                res.text().await?
            );
        }
        let res = res.bytes().await?;
        Ok(res.to_vec())
    }
}
