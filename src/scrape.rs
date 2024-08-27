use crate::{
    synthesis::Synthesis,
    task::{RunTask, Task, TaskRepo},
    voicevox_client::VoiceVoxSpeaker,
    Ctx,
};
use scraper::{Html, Selector};
use std::path::PathBuf;
use surrealdb::opt::RecordId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Episode {
    title: String,
    content: String,
}

impl Episode {
    pub(crate) fn new(title: String, content: String) -> Self {
        Self { title, content }
    }
}

pub(crate) struct EpisodeConverter {
    client: reqwest::Client,
}

static USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36";

impl EpisodeConverter {
    fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .unwrap(),
        }
    }

    fn get_title(&self, html: &Html) -> anyhow::Result<String> {
        let title_selector = Selector::parse("title").unwrap();
        let mut title = html.select(&title_selector);
        let Some(title) = title.next() else {
            return Err(anyhow::anyhow!("No title found"));
        };
        let title = title
            .text()
            .next()
            .map(|s| s.to_string())
            .ok_or(anyhow::anyhow!("No title found"))?;
        Ok(title)
    }

    fn get_content(&self, html: &Html) -> anyhow::Result<String> {
        let content_selector = Selector::parse("body").unwrap();
        let mut content = html.select(&content_selector);
        let Some(content) = content.next() else {
            return Err(anyhow::anyhow!("No content found"));
        };
        let content_html = content.html();
        let content = html2text::from_read(content_html.as_bytes(), 80)
            .split('\n')
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        Ok(content)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct ScrapeEpisode {
    #[serde(skip_serializing)]
    pub(crate) id: RecordId,
    url: String,
}

impl ScrapeEpisode {
    pub(crate) fn new(id: RecordId, url: String) -> Self {
        Self { id, url }
    }
}

impl RunTask for ScrapeEpisode {
    fn id(&self) -> &RecordId {
        &self.id
    }

    async fn run(&mut self, _ctx: &Ctx) -> anyhow::Result<Option<Task>> {
        let _span = tracing::debug_span!("run", id = self.id.id.to_string());
        let dir = PathBuf::from("temp");
        let scraper = EpisodeConverter::new();
        let res = scraper.client.get(&self.url).send().await?;
        if res.status() != reqwest::StatusCode::OK {
            anyhow::bail!("Failed to fetch: {}", res.status());
        }
        let html = res.text().await?;
        let html = Html::parse_document(&html);
        let title = scraper.get_title(&html)?;
        let content = scraper.get_content(&html)?;
        let episode = Episode::new(title, content);
        // let path = dir.join(format!("{}.json", self.id.id.to_string()));
        // let json = serde_json::to_string_pretty(&episode)?;
        // std::fs::write(&path, json)?;

        let task = Task::Synthesis(Synthesis {
            id: TaskRepo::new_id(),
            text: episode.content,
            speaker: VoiceVoxSpeaker::ZundaNormal,
            out: dir.join(format!("{}.wav", self.id.id.to_string())),
            artifacts: Vec::new(),
        });
        Ok(Some(task))
    }
}

#[cfg(test)]
mod tests {
    use crate::scrape::EpisodeConverter;
    use scraper::Html;
    use std::{fs::File, io::Read, path::PathBuf};

    fn read_html(path: &str) -> anyhow::Result<Html> {
        let mut f = File::open(PathBuf::from(path))?;
        let mut html = String::new();
        f.read_to_string(&mut html)?;
        let html = Html::parse_document(&html);
        Ok(html)
    }

    #[test]
    fn test_get_title() -> anyhow::Result<()> {
        let html = read_html("narou.html")?;
        let scraper = EpisodeConverter::new();
        let title = scraper.get_title(&html)?;
        println!("{}", title);
        Ok(())
    }

    #[test]
    fn test_get_content() -> anyhow::Result<()> {
        let html = read_html("narou.html")?;
        let scraper = EpisodeConverter::new();
        let content = scraper.get_content(&html)?;
        println!("{}", content);
        Ok(())
    }
}
