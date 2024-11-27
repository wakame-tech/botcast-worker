use super::{script_service::ScriptService, task_service::new_task};
use crate::{
    error::Error,
    model::{Args, Manuscript, Section},
    r2_storage::Storage,
};
use anyhow::{Context, Result};
use audio_generator::{
    generate_audio::{generate_audio, Sentence, SynthesisResult},
    workdir::WorkDir,
};
use chrono::Utc;
use repos::repo::{EpisodeRepo, PodcastRepo};
use repos::urn::Urn;
use repos::{
    entity::{Episode, EpisodeId, Podcast, PodcastId, ScriptId, Task},
    repo::ScriptRepo,
};
use std::{collections::BTreeMap, fs::File, io::Read, str::FromStr, sync::Arc};
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct EpisodeService {
    podcast_repo: Arc<dyn PodcastRepo>,
    script_repo: Arc<dyn ScriptRepo>,
    episode_repo: Arc<dyn EpisodeRepo>,
    storage: Arc<dyn Storage>,
    script_service: ScriptService,
}

fn new_episode(podcast: &Podcast, title: String, sections: Vec<Section>) -> Episode {
    Episode {
        id: Uuid::new_v4(),
        user_id: podcast.user_id,
        title,
        sections: serde_json::to_value(sections).unwrap(),
        podcast_id: podcast.id,
        audio_url: None,
        srt_url: None,
        created_at: Utc::now(),
    }
}

fn new_sentences(sections: Vec<Section>) -> Result<Vec<Sentence>, Error> {
    let mut sentences = vec![];
    for section in sections.iter() {
        match section {
            Section::Serif { text, speaker } => {
                let Urn::Other(resource, speaker_id) = speaker
                    .parse()
                    .with_context(|| format!("Invalid urn: {}", speaker))
                    .map_err(Error::Other)?
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
        podcast_repo: Arc<dyn PodcastRepo>,
        script_repo: Arc<dyn ScriptRepo>,
        episode_repo: Arc<dyn EpisodeRepo>,
        storage: Arc<dyn Storage>,
        script_service: ScriptService,
    ) -> Self {
        Self {
            podcast_repo: podcast_repo.clone(),
            script_repo: script_repo.clone(),
            episode_repo: episode_repo.clone(),
            storage,
            script_service,
        }
    }

    pub(crate) async fn generate_manuscript(
        &self,
        podcast_id: &PodcastId,
    ) -> anyhow::Result<Manuscript, Error> {
        let podcast = self.podcast_repo.find_by_id(podcast_id).await?;
        let script = self
            .script_repo
            .find_by_id(&ScriptId(podcast.script_id))
            .await?;
        let context = BTreeMap::from_iter([(
            "self".to_string(),
            serde_json::Value::String(format!("urn:podcast:{}", podcast.id)),
        )]);
        let manuscript: Manuscript = serde_json::from_value(
            self.script_service
                .run_template(&script.template, context)
                .await?,
        )
        .context("evaluated script is not ManuScript")
        .map_err(Error::Other)?;
        Ok(manuscript)
    }

    pub(crate) async fn new_episode_from_template(
        &self,
        podcast_id: &PodcastId,
    ) -> anyhow::Result<Option<Task>, Error> {
        let podcast = self.podcast_repo.find_by_id(podcast_id).await?;
        let manuscript: Manuscript = self.generate_manuscript(podcast_id).await?;
        let episode = new_episode(&podcast, manuscript.title, manuscript.sections);

        self.episode_repo.create(&episode).await?;

        let task = if let Some(cron) = podcast.cron {
            let next = cron::Schedule::from_str(&cron)
                .context("Invalid cron")
                .map_err(Error::Other)?
                .upcoming(Utc)
                .next()
                .context("Failed to get next cron")
                .map_err(Error::Other)?;

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
