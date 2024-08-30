use serde_json::Value;
use std::path::PathBuf;
use tokio::{fs, io::AsyncWriteExt};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) enum VoiceVoxSpeaker {
    ZundaNormal,
}

impl VoiceVoxSpeaker {
    fn id(&self) -> &str {
        match self {
            Self::ZundaNormal => "3",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct VoiceVox {
    endpoint: String,
    client: reqwest::Client,
}

impl Default for VoiceVox {
    fn default() -> Self {
        Self::new("http://localhost:50021".to_string())
    }
}

impl VoiceVox {
    pub(crate) fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: reqwest::Client::new(),
        }
    }

    pub(crate) async fn version(&self) -> anyhow::Result<String> {
        let url = format!("{}/version", self.endpoint);
        let res = self.client.get(url).send().await?;
        let version = res.text().await?;
        Ok(version)
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
        out: &PathBuf,
    ) -> anyhow::Result<()> {
        let url = format!("{}/synthesis?speaker={}", self.endpoint, speaker.id());
        let res = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&query)
            .send()
            .await?;
        if res.status() != reqwest::StatusCode::OK {
            anyhow::bail!(
                "Failed to synthesis: {} {}",
                res.status(),
                res.text().await?
            );
        }
        let res = res.bytes().await?;
        let mut f = fs::File::create(out).await?;
        f.write_all(&res).await?;
        Ok(())
    }
}
