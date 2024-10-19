use crate::r2_storage::{storage, Storage};
use crate::{
    episode::{
        script_service::{script_service, ScriptService},
        Manuscript, Section,
    },
    task::model::{Args, Task, TaskStatus},
};
use audio_generator::{
    generate_audio::{generate_audio, Sentence, SynthesisResult},
    workdir::WorkDir,
};
use chrono::Utc;
use repos::entity::Episode;
use repos::podcast_repo;
use repos::{
    episode_repo,
    repo::{EpisodeRepo, PodcastRepo, ScriptRepo},
    script_repo,
};
use script_runtime::parse_urn;
use std::{fs::File, io::Read, str::FromStr, sync::Arc};
use uuid::Uuid;

pub(crate) fn episode_service() -> EpisodeService {
    EpisodeService {
        podcast_repo: podcast_repo(),
        episode_repo: episode_repo(),
        script_repo: script_repo(),
        storage: storage(),
        script_service: script_service(),
    }
}

#[derive(Clone)]
pub(crate) struct EpisodeService {
    pub(crate) podcast_repo: Arc<dyn PodcastRepo>,
    pub(crate) episode_repo: Arc<dyn EpisodeRepo>,
    pub(crate) script_repo: Arc<dyn ScriptRepo>,
    pub(crate) storage: Arc<dyn Storage>,
    pub(crate) script_service: ScriptService,
}

impl EpisodeService {
    pub(crate) async fn new_episode(&self, pre_episode_id: Uuid) -> anyhow::Result<Task> {
        let (pre_episode, _) = self
            .episode_repo
            .find_by_id(&pre_episode_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Pre episode not found"))?;
        let podcast = self
            .podcast_repo
            .find_by_id(&pre_episode.podcast_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Podcast not found"))?;
        let Some(cron) = podcast.cron else {
            return Err(anyhow::anyhow!("Cron not found"));
        };

        let manuscript: Manuscript = serde_json::from_value(
            self.script_service
                .evaluate_script(pre_episode.script_id)
                .await?,
        )?;

        let episode = Episode {
            id: Uuid::new_v4(),
            user_id: pre_episode.user_id,
            title: manuscript.title,
            podcast_id: pre_episode.podcast_id,
            script_id: pre_episode.script_id,
            audio_url: None,
            srt_url: None,
            created_at: Utc::now(),
        };
        self.episode_repo.create(&episode).await?;

        let next = cron::Schedule::from_str(&cron)?
            .upcoming(Utc)
            .next()
            .expect("next");
        let task = Task {
            id: Uuid::new_v4(),
            status: TaskStatus::Pending,
            args: serde_json::to_value(Args::NewEpisode {
                pre_episode_id: episode.id,
            })?,
            execute_after: next,
            executed_at: None,
        };
        Ok(task)
    }

    pub(crate) async fn generate_audio(
        &self,
        work_dir: &WorkDir,
        episode_id: Uuid,
    ) -> anyhow::Result<()> {
        let (mut episode, _) = self
            .episode_repo
            .find_by_id(&episode_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Episode not found"))?;
        let script = self
            .script_repo
            .find_by_id(&episode.script_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Script not found"))?;
        let Some(result) = script.result.clone() else {
            return Err(anyhow::anyhow!("Manuscript not found"));
        };
        let manuscript: Manuscript = serde_json::from_value(result)?;

        let mut sentences = vec![];
        for section in manuscript.sections.iter() {
            match section {
                Section::Serif { text, speaker } => {
                    let (resource, speaker_id) = parse_urn(speaker)?;
                    for sentence in text.split(['\n', '。']) {
                        let sentence = sentence.trim();
                        if sentence.is_empty() {
                            continue;
                        }
                        sentences.push(Sentence::new(
                            resource.clone(),
                            speaker_id.clone(),
                            text.to_string(),
                        ));
                    }
                }
            }
        }
        let SynthesisResult { out_path, srt, .. } = generate_audio(work_dir, &sentences).await?;

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
        episode.srt_url = Some(format!("{}/{}", self.storage.get_endpoint(), srt_path));

        self.episode_repo.update(&episode).await?;
        Ok(())
    }
}
