use crate::AudioGenerator;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use tracing::instrument;

#[async_trait]
impl AudioGenerator for VoiceVoxClient {
    #[instrument(skip(self))]
    async fn generate(&self, speaker_id: &str, text: &str) -> Result<Vec<u8>> {
        let query = self.query(text, speaker_id).await?;
        let audio = self.synthesis(query, speaker_id).await?;
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

    #[instrument(skip(self))]
    pub(crate) async fn query(&self, text: &str, speaker: &str) -> anyhow::Result<Value> {
        let text = urlencoding::encode(text);
        let url = format!(
            "{}/audio_query?text={}&speaker={}",
            self.endpoint, text, speaker,
        );
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

    #[instrument(skip(self, query))]
    pub(crate) async fn synthesis(&self, query: Value, speaker: &str) -> anyhow::Result<Vec<u8>> {
        let url = format!("{}/synthesis?speaker={}", self.endpoint, speaker);
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
