use encoding::{all::UTF_8, DecoderTrap, Encoding};
use std::time::Duration;

pub struct HttpClient {
    client: reqwest::Client,
}

impl HttpClient {
    pub fn new(user_agent: Option<String>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(user_agent.unwrap_or_default())
                .timeout(Duration::from_secs(5))
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    pub async fn fetch_content_as_utf8(&self, url: String) -> anyhow::Result<String> {
        let res = self.client.get(url).send().await?;
        if res.status() != reqwest::StatusCode::OK {
            anyhow::bail!("Failed to fetch: {}", res.status());
        }
        let html = res.bytes().await?;
        let html = match xmldecl::parse(&html) {
            Some(e) => e.decode(&html).0.into_owned(),
            None => UTF_8
                .decode(&html, DecoderTrap::Strict)
                .map_err(|e| anyhow::anyhow!("Failed to decode: {}", e))?,
        };
        Ok(html)
    }
}
