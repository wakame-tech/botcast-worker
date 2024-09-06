use super::workdir::WorkDir;
use serde_json::Value;
use std::{fs::OpenOptions, io::Write, path::PathBuf};
use tokio::{fs, io::AsyncWriteExt, process::Command};

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

pub(crate) async fn concat_wavs(work_dir: &WorkDir, paths: &[PathBuf]) -> anyhow::Result<PathBuf> {
    let inputs_path = work_dir.dir().join("inputs.txt");
    let text = paths
        .iter()
        .map(|p| format!("file '{}'", p.file_name().unwrap().to_string_lossy()))
        .collect::<Vec<_>>()
        .join("\n");
    let mut f = OpenOptions::new()
        .truncate(true)
        .write(true)
        .create(true)
        .open(&inputs_path)?;
    f.write_all(text.as_bytes())?;

    let episode_wav_path = work_dir.dir().join("episode.wav");
    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        "-y",
        "-f",
        "concat",
        "-i",
        inputs_path.display().to_string().as_str(),
        episode_wav_path.display().to_string().as_str(),
    ]);
    let res = cmd.output().await?;
    if !res.status.success() {
        anyhow::bail!("Failed to concat wavs: {}", String::from_utf8(res.stderr)?);
    }
    Ok(episode_wav_path)
}
