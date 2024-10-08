use super::EpisodeRepo;
use crate::{episode::use_work_dir, infra::http_client::HttpClient};
use anyhow::{anyhow, Context};
use readable_text::{html2md::Html2MdExtractor, Extractor};
use reqwest::Url;
use std::{fs::File, io::Write, sync::Arc};
use uuid::Uuid;

pub(crate) struct ScrapeService {
    pub(crate) episode_repo: Arc<dyn EpisodeRepo>,
}

impl ScrapeService {
    pub(crate) async fn generate_script_from_url(
        &self,
        task_id: Uuid,
        episode_id: Uuid,
        url: Url,
    ) -> anyhow::Result<Vec<String>> {
        let work_dir = use_work_dir(&task_id)?;
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
}
