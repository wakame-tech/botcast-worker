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
use repos::entity::{Episode, EpisodeId, Podcast, PodcastId, ScriptId, Task};
use repos::provider::{ProvideEpisodeRepo, ProvidePodcastRepo, ProvideScriptRepo};
use repos::repo::{EpisodeRepo, PodcastRepo, ScriptRepo};
use repos::urn::Urn;
use std::{collections::BTreeMap, fs::File, io::Read, str::FromStr, sync::Arc};
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct EpisodeService {
    podcast_repo: Arc<dyn PodcastRepo>,
    episode_repo: Arc<dyn EpisodeRepo>,
    script_repo: Arc<dyn ScriptRepo>,
    storage: Arc<dyn Storage>,
    script_service: ScriptService,
}

fn new_episode(podcast: &Podcast, title: String) -> Episode {
    Episode {
        id: Uuid::new_v4(),
        user_id: podcast.user_id,
        title,
        podcast_id: podcast.id,
        script_id: podcast.script_id,
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
                        sentence.to_string(),
                    ));
                }
            }
        }
    }
    Ok(sentences)
}

impl EpisodeService {
    pub(crate) fn new(
        provider: impl ProvidePodcastRepo + ProvideEpisodeRepo + ProvideScriptRepo + ProviderStorage,
    ) -> Self {
        Self {
            podcast_repo: provider.podcast_repo(),
            episode_repo: provider.episode_repo(),
            script_repo: provider.script_repo(),
            storage: provider.storage(),
            script_service: ScriptService::new(provider),
        }
    }

    pub(crate) async fn generate_manuscript(
        &self,
        podcast_id: &PodcastId,
    ) -> anyhow::Result<Manuscript, Error> {
        let podcast = self.podcast_repo.find_by_id(podcast_id).await?;
        let context_values = BTreeMap::from_iter([(
            "self".to_string(),
            serde_json::Value::String(format!("urn:podcast:{}", podcast.id)),
        )]);
        let manuscript: Manuscript = serde_json::from_value(
            self.script_service
                .evaluate_script(&ScriptId(podcast.script_id), context_values)
                .await?,
        )
        .map_err(|e| Error::Other(anyhow::anyhow!("evaluated script is not ManuScript: {}", e)))?;
        Ok(manuscript)
    }

    pub(crate) async fn new_episode_from_template(
        &self,
        podcast_id: &PodcastId,
    ) -> anyhow::Result<Option<Task>, Error> {
        let podcast = self.podcast_repo.find_by_id(podcast_id).await?;
        let manuscript: Manuscript = self.generate_manuscript(podcast_id).await?;
        let episode = new_episode(&podcast, manuscript.title);

        self.episode_repo.create(&episode).await?;

        let task = if let Some(cron) = podcast.cron {
            let next = cron::Schedule::from_str(&cron)
                .map_err(|e| Error::Other(anyhow::anyhow!("Invalid cron: {}", e)))?
                .upcoming(Utc)
                .next()
                .ok_or_else(|| Error::Other(anyhow::anyhow!("Failed to get next cron")))?;

            let task = new_task(
                Args::NewEpisode {
                    podcast_id: podcast_id.clone(),
                },
                next,
            );
            Some(task)
        } else {
            None
        };
        Ok(task)
    }

    pub(crate) async fn generate_audio(
        &self,
        work_dir: &WorkDir,
        episode_id: &EpisodeId,
    ) -> anyhow::Result<(), Error> {
        let (mut episode, _) = self.episode_repo.find_by_id(episode_id).await?;
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

        let audio_path = format!("episodes/{}.mp3", episode.id.hyphenated());
        self.storage
            .upload(&audio_path, &audio, "audio/mp3")
            .await
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to upload audio: {}", e)))?;
        episode.audio_url = Some(audio_path);

        let srt_path = format!("episodes/{}.srt", episode.id.hyphenated());
        self.storage
            .upload(&srt_path, srt.as_bytes(), "text/plain")
            .await
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to upload srt: {}", e)))?;
        episode.srt_url = Some(srt_path);

        self.episode_repo.update(&episode).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Local};
    use cron::Schedule;

    #[test]
    fn test_cron() -> anyhow::Result<()> {
        // every monday at 9:00 UTC = 18:00 JST
        let schedule = Schedule::from_str("0 0 9 * * Mon")?;
        let next = schedule.upcoming(Utc).next().unwrap();
        println!("{:?}", DateTime::<Local>::from(next));
        Ok(())
    }
}
