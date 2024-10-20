use crate::r2_storage::{storage, Storage};
use crate::{
    episode::{
        script_service::{script_service, ScriptService},
        Manuscript, Section,
    },
    task::Args,
};
use audio_generator::{
    generate_audio::{generate_audio, Sentence, SynthesisResult},
    workdir::WorkDir,
};
use chrono::Utc;
use repos::entity::{Episode, EpisodeId, PodcastId, ScriptId, Task, TaskStatus};
use repos::podcast_repo;
use repos::urn::Urn;
use repos::{
    episode_repo,
    repo::{EpisodeRepo, PodcastRepo, ScriptRepo},
    script_repo,
};
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

impl EpisodeService {
    pub(crate) async fn new_episode(&self, pre_episode_id: &EpisodeId) -> anyhow::Result<Task> {
        let (pre_episode, _) = self.episode_repo.find_by_id(&pre_episode_id).await?;
        let podcast = self
            .podcast_repo
            .find_by_id(&PodcastId(pre_episode.podcast_id))
            .await?;
        let Some(cron) = podcast.cron else {
            return Err(anyhow::anyhow!("Cron not found"));
        };

        let manuscript: Manuscript = serde_json::from_value(
            self.script_service
                .evaluate_script(&ScriptId(pre_episode.script_id))
                .await?,
        )?;

        let episode = new_episode(&pre_episode, manuscript.title);
        self.episode_repo.create(&episode).await?;

        let next = cron::Schedule::from_str(&cron)?
            .upcoming(Utc)
            .next()
            .expect("next");
        let task = Task {
            id: Uuid::new_v4(),
            status: TaskStatus::Pending,
            args: serde_json::to_value(Args::NewEpisode {
                pre_episode_id: EpisodeId(episode.id),
            })?,
            execute_after: next,
            executed_at: None,
        };
        Ok(task)
    }

    pub(crate) async fn generate_audio(
        &self,
        work_dir: &WorkDir,
        episode_id: &EpisodeId,
    ) -> anyhow::Result<()> {
        let (mut episode, _) = self.episode_repo.find_by_id(&episode_id).await?;
        let script = self
            .script_repo
            .find_by_id(&ScriptId(episode.script_id))
            .await?;
        let Some(result) = script.result.clone() else {
            return Err(anyhow::anyhow!("Manuscript not found"));
        };
        let manuscript: Manuscript = serde_json::from_value(result)?;

        let mut sentences = vec![];
        for section in manuscript.sections.iter() {
            match section {
                Section::Serif { text, speaker } => {
                    let Urn::Other(resource, speaker_id) = speaker.parse()? else {
                        return Err(anyhow::anyhow!("Invalid urn"));
                    };
                    for sentence in text.split(['\n', '。']) {
                        let sentence = sentence.trim();
                        if sentence.is_empty() {
                            continue;
                        }
                        sentences.push(Sentence::new(
                            (resource.clone(), speaker_id.to_string().parse()?),
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
