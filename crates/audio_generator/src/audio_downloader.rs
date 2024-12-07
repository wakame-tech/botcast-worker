use crate::{
    ffmpeg::slice_audio, generate_audio::SectionSegment, workdir::WorkDir, AudioGenerator,
};
use async_trait::async_trait;

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
        work_dir: &WorkDir,
        segment: SectionSegment,
    ) -> anyhow::Result<Vec<u8>> {
        let SectionSegment::Audio {
            url,
            from_sec,
            to_sec,
        } = segment
        else {
            return Err(anyhow::anyhow!("Invalid segment"));
        };
        let (from_sec, duration_sec) = match (from_sec, to_sec) {
            (Some(from_sec), Some(to_sec)) if from_sec < to_sec => (from_sec, to_sec - from_sec),
            _ => {
                return Err(anyhow::anyhow!("Invalid from_sec and to_sec"));
            }
        };
        let response = self.client.get(url).send().await?;
        let audio = response.bytes().await?;
        let audio_file_path = work_dir.dir().join("tmp.mp3");
        let sliced_audio_file_path = work_dir.dir().join("tmp_sliced.mp3");
        std::fs::write(&audio_file_path, &audio)?;
        slice_audio(
            &audio_file_path,
            &sliced_audio_file_path,
            from_sec,
            duration_sec,
        )
        .await?;
        let sliced = std::fs::read(&sliced_audio_file_path)?;
        Ok(sliced)
    }
}
