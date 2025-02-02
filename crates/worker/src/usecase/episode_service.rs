use crate::{error::Error, r2_storage::Storage};
use anyhow::Context;
use api::episode::Section;
use audio_generator::{
    generate_audio::{generate_audio, SynthesisResult},
    workdir::WorkDir,
};
use repos::entity::EpisodeId;
use repos::repo::EpisodeRepo;
use std::{fs::File, io::Read, sync::Arc};
use tracing::instrument;

/// evaluated script
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Manuscript {
    title: String,
    sections: Vec<Section>,
}

#[derive(Clone)]
pub(crate) struct EpisodeService {
    episode_repo: Arc<dyn EpisodeRepo>,
    storage: Arc<dyn Storage>,
}

impl EpisodeService {
    pub(crate) fn new(episode_repo: Arc<dyn EpisodeRepo>, storage: Arc<dyn Storage>) -> Self {
        Self {
            episode_repo: episode_repo.clone(),
            storage,
        }
    }

    #[instrument(skip(self, work_dir), ret)]
    pub(crate) async fn generate_audio(
        &self,
        work_dir: &WorkDir,
        episode_id: &EpisodeId,
    ) -> anyhow::Result<(), Error> {
        let mut episode = self.episode_repo.find_by_id(episode_id).await?;
        let sections: Vec<Section> = serde_json::from_value(episode.sections.clone())
            .context("Failed to parse sections")
            .map_err(Error::Other)?;
        let SynthesisResult {
            out_path,
            srt,
            duration_sec,
        } = generate_audio(work_dir, sections)
            .await
            .context("Failed to generate audio")
            .map_err(Error::Other)?;

        episode.duration_sec = Some(duration_sec.round() as i32);
        let mut file = File::open(&out_path)
            .context("Failed to open audio file")
            .map_err(Error::Other)?;
        let mut audio = vec![];
        file.read_to_end(&mut audio)
            .context("Failed to read audio file")
            .map_err(Error::Other)?;

        let audio_path = format!("episodes/{}.mp3", episode.id.hyphenated());
        self.storage
            .upload(&audio_path, &audio, "audio/mp3")
            .await
            .context("Failed to upload audio")
            .map_err(Error::Other)?;
        episode.audio_url = Some(audio_path);

        let srt_path = format!("episodes/{}.srt", episode.id.hyphenated());
        self.storage
            .upload(&srt_path, srt.as_bytes(), "text/plain")
            .await
            .context("Failed to upload srt")
            .map_err(Error::Other)?;
        episode.srt_url = Some(srt_path);

        self.episode_repo.update(&episode).await?;
        Ok(())
    }
}
