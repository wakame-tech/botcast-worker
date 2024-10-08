use super::{use_work_dir, EpisodeRepo};
use crate::infra::{
    voicevox_synthesizer::{AudioSynthesizer, SynthesisResult},
    Storage,
};
use std::{fs::File, io::Read, sync::Arc};
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct EpisodeService {
    pub(crate) episode_repo: Arc<dyn EpisodeRepo>,
    pub(crate) storage: Arc<dyn Storage>,
    pub(crate) synthesizer: Arc<dyn AudioSynthesizer>,
}

impl EpisodeService {
    pub(crate) async fn synthesis_audio(
        &self,
        task_id: Uuid,
        episode_id: Uuid,
        sentences: Vec<String>,
    ) -> anyhow::Result<()> {
        let work_dir = use_work_dir(&task_id)?;
        let Some(mut episode) = self.episode_repo.find_by_id(&episode_id).await? else {
            return Err(anyhow::anyhow!("Episode not found"));
        };

        let SynthesisResult { out_path, srt, .. } = self
            .synthesizer
            .synthesis_sentences(&work_dir, sentences)
            .await?;

        self.episode_repo.update(&episode).await?;

        let mut file = File::open(&out_path)?;
        let mut audio = vec![];
        file.read_to_end(&mut audio)?;

        let mp3_path = format!("episodes/{}.mp3", episode.id.hyphenated());
        self.storage.upload(&mp3_path, &audio, "audio/mp3").await?;
        episode.audio_url = Some(format!("{}/{}", self.storage.get_endpoint(), mp3_path));

        let srt_path = format!("episodes/{}.srt", episode.id.hyphenated());
        self.storage
            .upload(&srt_path, srt.as_bytes(), "text/plain")
            .await?;
        episode.script_url = Some(format!("{}/{}", self.storage.get_endpoint(), srt_path));

        self.episode_repo.update(&episode).await?;
        Ok(())
    }
}
