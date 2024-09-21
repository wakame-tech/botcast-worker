use super::workdir::WorkDir;
use serde_json::Value;
use std::{fs::OpenOptions, io::Write, path::PathBuf, time::Duration};
use tokio::{fs, io::AsyncWriteExt, process::Command};
use wavers::Wav;

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

impl VoiceVox {
    pub(crate) fn new() -> Self {
        let endpoint =
            std::env::var("VOICEVOX_ENDPOINT").unwrap_or("http://localhost:50021".to_string());
        log::info!("VoiceVox endpoint: {}", endpoint);
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

pub(crate) fn get_duration(wav: &Wav<i16>) -> Duration {
    let data_size = wav.header().data().size;

    let sample_rate = wav.sample_rate() as u32;
    let n_channels = wav.n_channels() as u32;
    let bytes_per_sample = (wav.header().fmt_chunk.bits_per_sample / 8) as u32;

    Duration::from_secs_f32(data_size as f32 / (sample_rate * n_channels * bytes_per_sample) as f32)
}

pub(crate) async fn concat_audios(
    work_dir: &WorkDir,
    paths: &[PathBuf],
) -> anyhow::Result<PathBuf> {
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

    let episode_audio_path = work_dir.dir().join("episode.mp3");
    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        "-y",
        "-f",
        "concat",
        "-i",
        inputs_path.display().to_string().as_str(),
        "-vn",
        "-ar",
        "44100",
        "-ac",
        "2",
        "-b:a",
        "192k",
        episode_audio_path.display().to_string().as_str(),
    ]);
    let res = cmd.output().await?;
    if !res.status.success() {
        anyhow::bail!(
            "Failed to concat audios: {}",
            String::from_utf8(res.stderr)?
        );
    }
    Ok(episode_audio_path)
}