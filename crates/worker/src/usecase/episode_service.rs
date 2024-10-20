use super::{script_service::ScriptService, task_service::new_task};
use crate::{
    error::Error,
    model::{Args, Manuscript, Section},
    r2_storage::{ProviderStorage, Storage},
};
use anyhow::Result;
use audio_generator::{
    generate_audio::{generate_audio, Sentence, SynthesisResult},
    workdir::WorkDir,
};
use chrono::Utc;
use repos::entity::{Episode, EpisodeId, PodcastId, ScriptId, Task};
use repos::provider::{ProvideEpisodeRepo, ProvidePodcastRepo, ProvideScriptRepo, Provider};
use repos::repo::{EpisodeRepo, PodcastRepo, ScriptRepo};
use repos::urn::Urn;
use std::{fs::File, io::Read, str::FromStr, sync::Arc};
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct EpisodeService {
    podcast_repo: Arc<dyn PodcastRepo>,
    episode_repo: Arc<dyn EpisodeRepo>,
    script_repo: Arc<dyn ScriptRepo>,
    storage: Arc<dyn Storage>,
    script_service: ScriptService,
}

fn new_episode(pre_episode: &Episode, title: String) -> Episode {
    Episode {
        id: Uuid::new_v4(),
        user_id: pre_episode.user_id,
        title,
        podcast_id: pre_episode.podcast_id,
        script_id: pre_episode.script_id,
        audio_url: None,
        srt_url: None,
        created_at: Utc::now(),
    }
}

fn new_sentences(manuscript: Manuscript) -> Result<Vec<Sentence>, Error> {
    let mut sentences = vec![];
    for section in manuscript.sections.iter() {
        match section {
            Section::Serif { text, speaker } => {
                let Urn::Other(resource, speaker_id) = speaker
                    .parse()
                    .map_err(|e| Error::Other(anyhow::anyhow!("Invalid urn: {}", e)))?
                else {
                    return Err(Error::Other(anyhow::anyhow!("Invalid urn: {}", speaker)));
                };
                for sentence in text.split(['\n', '。']) {
                    let sentence = sentence.trim();
                    if sentence.is_empty() {
                        continue;
                    }
                    sentences.push(Sentence::new(
                        (resource.clone(), speaker_id.to_string()),
                        text.to_string(),
                    ));
                }
            }
        }
    }
    Ok(sentences)
}

impl EpisodeService {
    pub(crate) fn new(provider: Provider) -> Self {
        Self {
            podcast_repo: provider.podcast_repo(),
            episode_repo: provider.episode_repo(),
            script_repo: provider.script_repo(),
            storage: provider.storage(),
            script_service: ScriptService::new(provider),
        }
    }

    pub(crate) async fn new_episode(
        &self,
        pre_episode_id: &EpisodeId,
    ) -> anyhow::Result<Task, Error> {
        let (pre_episode, _) = self.episode_repo.find_by_id(&pre_episode_id).await?;
        let podcast = self
            .podcast_repo
            .find_by_id(&PodcastId(pre_episode.podcast_id))
            .await?;
        let Some(cron) = podcast.cron else {
            return Err(Error::Other(anyhow::anyhow!("Cron not found")));
        };

        let manuscript: Manuscript = serde_json::from_value(
            self.script_service
                .evaluate_script(&ScriptId(pre_episode.script_id))
                .await?,
        )
        .map_err(|e| Error::Other(anyhow::anyhow!("evaluated script is not ManuScript: {}", e)))?;

        let episode = new_episode(&pre_episode, manuscript.title);
        self.episode_repo.create(&episode).await?;

        let next = cron::Schedule::from_str(&cron)
            .map_err(|e| Error::Other(anyhow::anyhow!("Invalid cron: {}", e)))?
            .upcoming(Utc)
            .next()
            .ok_or_else(|| Error::Other(anyhow::anyhow!("Failed to get next cron")))?;
        let task = new_task(
            Args::NewEpisode {
                pre_episode_id: EpisodeId(episode.id),
            },
            next,
        );
        Ok(task)
    }

    pub(crate) async fn generate_audio(
        &self,
        work_dir: &WorkDir,
        episode_id: &EpisodeId,
    ) -> anyhow::Result<(), Error> {
        let (mut episode, _) = self.episode_repo.find_by_id(&episode_id).await?;
        let script = self
            .script_repo
            .find_by_id(&ScriptId(episode.script_id))
            .await?;
        let Some(result) = script.result.clone() else {
            return Err(Error::Other(anyhow::anyhow!("Manuscript not found")));
        };
        let manuscript: Manuscript = serde_json::from_value(result).map_err(|e| {
            Error::Other(anyhow::anyhow!("evaluated script is not ManuScript: {}", e))
        })?;
        let sentences = new_sentences(manuscript)?;
        let SynthesisResult { out_path, srt, .. } = generate_audio(work_dir, &sentences)
            .await
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to generate audio: {}", e)))?;

        let mut file = File::open(&out_path)
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to open audio file: {}", e)))?;
        let mut audio = vec![];
        file.read_to_end(&mut audio)
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to read audio file: {}", e)))?;

        let mp3_path = format!("episodes/{}.mp3", episode.id.hyphenated());
        self.storage
            .upload(&mp3_path, &audio, "audio/mp3")
            .await
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to upload audio: {}", e)))?;
        episode.audio_url = Some(format!("{}/{}", self.storage.get_endpoint(), mp3_path));

        let srt_path = format!("episodes/{}.srt", episode.id.hyphenated());
        self.storage
            .upload(&srt_path, srt.as_bytes(), "text/plain")
            .await
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to upload srt: {}", e)))?;
        episode.srt_url = Some(format!("{}/{}", self.storage.get_endpoint(), srt_path));

        self.episode_repo.update(&episode).await?;
        Ok(())
    }
}
