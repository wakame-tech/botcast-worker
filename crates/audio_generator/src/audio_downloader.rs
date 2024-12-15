use std::path::PathBuf;

use crate::{ffmpeg::slice_audio, workdir::WorkDir, AudioGenerator};
use api::episode::Section;
use async_trait::async_trait;
use tokio::fs;

pub(crate) struct AudioDownloader {
    client: reqwest::Client,
}

impl AudioDownloader {
    pub(crate) fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl AudioGenerator for AudioDownloader {
    async fn generate(
        &self,
        i: &mut usize,
        work_dir: &WorkDir,
        section: Section,
    ) -> anyhow::Result<Vec<(PathBuf, String)>> {
        let Section::Audio { url, from, to } = section else {
            return Err(anyhow::anyhow!("Invalid segment"));
        };
        let response = self.client.get(url).send().await?;
        let audio = response.bytes().await?;
        let audio_file_path = work_dir.dir().join("tmp.mp3");
        std::fs::write(&audio_file_path, &audio)?;

        let sliced_audio_file_path = work_dir.dir().join(&format!("{}.wav", i));
        *i += 1;
        slice_audio(&audio_file_path, &sliced_audio_file_path, from, to).await?;
        fs::remove_file(&audio_file_path).await?;
        Ok(vec![(sliced_audio_file_path, "♪".to_string())])
    }
}
