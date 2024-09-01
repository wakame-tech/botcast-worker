use super::{extractor::Extractor, synthesis::Synthesis, Args, RunTask};
use crate::{api::ctx::Ctx, repo::EpisodeRepo};
use anyhow::Context;
use uuid::Uuid;

static USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Scrape {
    episode_id: String,
    url: String,
}

impl RunTask for Scrape {
    async fn run(&self, _task_id: Uuid, ctx: &Ctx) -> anyhow::Result<Option<Args>> {
        let repo = EpisodeRepo::new(ctx.pool.clone());
        let Some(mut episode) = repo.find_by_id(&Uuid::parse_str(&self.episode_id)?).await? else {
            return Err(anyhow::anyhow!("Episode not found"));
        };

        let client = reqwest::Client::builder().user_agent(USER_AGENT).build()?;
        let res = client.get(&self.url).send().await?;
        if res.status() != reqwest::StatusCode::OK {
            anyhow::bail!("Failed to fetch: {}", res.status());
        }
        let html = res.text().await?;

        let extractor = Extractor::new(html)?;
        let title = extractor.get_title().context("Failed to get title")?;
        let content = extractor.get_content().context("Failed to get content")?;
        log::info!("Scraped: {} {} B", episode.title, content.len());

        episode.title = title;
        episode.content = Some(content);
        repo.update(&episode).await?;

        Ok(Some(Args::Synthesis(Synthesis {
            episode_id: episode.id,
        })))
    }
}
