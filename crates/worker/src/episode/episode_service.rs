use super::EpisodeRepo;
use crate::infra::{http_client::HttpClient, workdir::WorkDir, Storage};
use anyhow::{anyhow, Context};
use axum::async_trait;
use reqwest::Url;
use scriper::{html2md::Html2MdExtractor, Extractor};
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};
use uuid::Uuid;

pub(crate) struct SynthesisResult {
    pub(crate) out_path: PathBuf,
    pub(crate) srt: String,
}

#[async_trait]
pub(crate) trait AudioSynthesizer: Send + Sync {
    async fn synthesis_sentences(
        &self,
        work_dir: &WorkDir,
        sentences: Vec<String>,
    ) -> anyhow::Result<SynthesisResult>;
}

pub(crate) struct EpisodeService {
    pub(crate) episode_repo: Box<dyn EpisodeRepo>,
    pub(crate) storage: Box<dyn Storage>,
    pub(crate) synthesizer: Box<dyn AudioSynthesizer>,
}

impl EpisodeService {
    fn use_work_dir(&self, task_id: &Uuid) -> anyhow::Result<WorkDir> {
        let keep = std::env::var("KEEP_WORKDIR")
            .unwrap_or("false".to_string())
            .parse()?;
        WorkDir::new(task_id, keep)
    }

    async fn generate_script_from_url(
        &self,
        task_id: Uuid,
        episode_id: Uuid,
        url: Url,
    ) -> anyhow::Result<Vec<String>> {
        let work_dir = self.use_work_dir(&task_id)?;
        let episode = self
            .episode_repo
            .find_by_id(&episode_id)
            .await?
            .ok_or_else(|| anyhow!("Episode not found"))?;

        let client = HttpClient::default();
        let html = client
            .fetch_content_as_utf8(url)
            .await
            .context("Failed to fetch content")?;
        let content = Html2MdExtractor::extract(&html).context("Failed to extract content")?;
        let mut content_file = File::create(work_dir.dir().join("content.md"))?;
        write!(content_file, "# {}\n\n", episode.title)?;
        content_file.write_all(content.as_bytes())?;

        log::info!("Scraped: {} {} B", episode.title, content.len());

        let sentences = content
            .split_inclusive(['。', '\n'])
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        if sentences.is_empty() {
            anyhow::bail!("Sentences is empty");
        }
        Ok(sentences)
    }

    async fn synthesis_audio(
        &self,
        task_id: Uuid,
        episode_id: Uuid,
        sentences: Vec<String>,
    ) -> anyhow::Result<()> {
        let work_dir = self.use_work_dir(&task_id)?;
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

    pub(crate) async fn run(
        &self,
        task_id: Uuid,
        episode_id: Uuid,
        url: Url,
    ) -> anyhow::Result<()> {
        let sentences = self
            .generate_script_from_url(task_id, episode_id, url)
            .await?;
        self.synthesis_audio(task_id, episode_id, sentences).await?;
        Ok(())
    }
}
