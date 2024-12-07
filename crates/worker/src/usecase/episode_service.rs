use crate::{error::Error, r2_storage::Storage};
use anyhow::{Context, Result};
use api::episode::Section;
use audio_generator::{
    generate_audio::{generate_audio, Sentence, SynthesisResult},
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

fn new_sentences(sections: Vec<Section>) -> Result<Vec<Sentence>, Error> {
    let mut sentences = vec![];
    for section in sections.iter() {
        match section {
            Section::Serif { text, speaker } => {
                for sentence in text.split(['\n', '。']) {
                    let sentence = sentence.trim();
                    if sentence.is_empty() {
                        continue;
                    }
                    sentences.push(Sentence::new(speaker.to_string(), sentence.to_string()));
                }
            }
        }
    }
    Ok(sentences)
}

impl EpisodeService {
    pub(crate) fn new(episode_repo: Arc<dyn EpisodeRepo>, storage: Arc<dyn Storage>) -> Self {
        Self {
            episode_repo: episode_repo.clone(),
            storage,
        }
    }

    #[instrument(skip(self, work_dir))]
    pub(crate) async fn generate_audio(
        &self,
        work_dir: &WorkDir,
        episode_id: &EpisodeId,
    ) -> anyhow::Result<(), Error> {
        let (mut episode, _) = self.episode_repo.find_by_id(episode_id).await?;
        let sections: Vec<Section> = serde_json::from_value(episode.sections.clone())
            .context("Failed to parse sections")
            .map_err(Error::Other)?;
        let sentences = new_sentences(sections)?;
        let SynthesisResult { out_path, srt, .. } = generate_audio(work_dir, &sentences)
            .await
            .context("Failed to generate audio")
            .map_err(Error::Other)?;

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
