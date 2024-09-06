use super::RunTask;
use crate::{api::ctx::Ctx, worker::USER_AGENT};
use anyhow::Context;
use scriper::extractor::HtmlExtractor;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Script {
    url: String,
}

impl RunTask for Script {
    async fn run_once(&self, _ctx: &Ctx) -> anyhow::Result<serde_json::Value> {
        let client = reqwest::Client::builder().user_agent(USER_AGENT).build()?;
        let res = client.get(&self.url).send().await?;
        if res.status() != reqwest::StatusCode::OK {
            anyhow::bail!("Failed to fetch: {}", res.status());
        }
        let html = res.text().await?;

        let extractor = HtmlExtractor::new(html)?;
        let title = extractor.get_title().context("Failed to get title")?;
        let content = extractor.get_content().context("Failed to get content")?;
        Ok(serde_json::json!({
            "title": title,
            "content": content,
        }))
    }
}
