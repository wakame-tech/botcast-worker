use super::{
    generate_audio::{generate_audio, SynthesisResult},
    script_service::{script_service, ScriptService},
};
use crate::infra::{
    r2_storage::{storage, Storage},
    workdir::WorkDir,
};
use repos::{
    episode_repo,
    repo::{EpisodeRepo, ScriptRepo},
    script_repo,
};
use script_runtime::Manuscript;
use std::{fs::File, io::Read, sync::Arc};
use uuid::Uuid;

pub fn episode_service() -> EpisodeService {
    EpisodeService {
        episode_repo: episode_repo(),
        script_repo: script_repo(),
        storage: storage(),
        script_service: script_service(),
    }
}

#[derive(Clone)]
pub(crate) struct EpisodeService {
    pub(crate) episode_repo: Arc<dyn EpisodeRepo>,
    pub(crate) script_repo: Arc<dyn ScriptRepo>,
    pub(crate) storage: Arc<dyn Storage>,
    pub(crate) script_service: ScriptService,
}

impl EpisodeService {
    pub(crate) async fn generate_manuscript(&self, episode_id: Uuid) -> anyhow::Result<()> {
        // let work_dir = use_work_dir(&task_id)?;
        let mut episode = self
            .episode_repo
            .find_by_id(&episode_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Episode not found"))?;
        let script = self
            .script_repo
            .find_by_id(&episode.script_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Script not found"))?;
        let manuscript = self
            .script_service
            .evaluate_to_manuscript(script.template)
            .await?;

        episode.manuscript = Some(serde_json::to_value(manuscript)?);
        self.episode_repo.update(&episode).await?;
        Ok(())
    }

    pub(crate) async fn synthesis_audio(
        &self,
        work_dir: &WorkDir,
        episode_id: Uuid,
    ) -> anyhow::Result<()> {
        let mut episode = self
            .episode_repo
            .find_by_id(&episode_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Episode not found"))?;

        let Some(manuscript) = episode.manuscript.clone() else {
            return Err(anyhow::anyhow!("Manuscript not found"));
        };
        let manuscript: Manuscript = serde_json::from_value(manuscript)?;
        episode.title = manuscript.title.clone();

        let SynthesisResult { out_path, srt, .. } = generate_audio(work_dir, &manuscript).await?;

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

        self.episode_repo.update(&episode).await?;
        Ok(())
    }
}
