use serde_json::Value;
use tracing::instrument;

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

    async fn _version(&self) -> anyhow::Result<Value> {
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
        let encoded = urlencoding::encode(text);
        let url = format!(
            "{}/audio_query?text={}&speaker={}",
            self.endpoint, encoded, speaker,
        );
        let res = self.client.post(&url).send().await?;
        if res.status() != reqwest::StatusCode::OK {
            anyhow::bail!(
                "Failed to query {}: text={}, {} {}",
                url,
                text,
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
        let res = self.client.post(&url).json(&query).send().await?;
        if res.status() != reqwest::StatusCode::OK {
            anyhow::bail!(
                "Failed to synthesis {}: {} {}",
                url,
                res.status(),
                res.text().await?
            );
        }
        let res = res.bytes().await?;
        Ok(res.to_vec())
    }
}
