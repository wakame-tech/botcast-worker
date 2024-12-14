use crate::{imports::as_string, runtime::evaluate_args};
use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use json_e::{
    value::{AsyncCallable, Value},
    Context,
};
use rss::{Channel, Item};
use std::str::FromStr;
use tracing::instrument;

#[derive(Debug, serde::Serialize)]
struct RssFeed {
    title: String,
    description: String,
    items: Vec<RssFeedItem>,
}

impl TryFrom<&Item> for RssFeedItem {
    type Error = anyhow::Error;

    fn try_from(item: &Item) -> Result<Self> {
        Ok(Self {
            pub_date: item
                .pub_date()
                .map(|d| {
                    DateTime::parse_from_rfc2822(d)
                        .map_err(|e| anyhow::anyhow!("Failed to parse date: {}", e))
                })
                .transpose()?
                .unwrap_or_default(),
            title: item.title().unwrap_or_default().to_string(),
            description: item.description().unwrap_or_default().to_string(),
            link: item.link().unwrap_or_default().to_string(),
        })
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct RssFeedItem {
    title: String,
    description: String,
    pub_date: DateTime<FixedOffset>,
    link: String,
}

#[derive(Clone)]
pub(crate) struct Rss;

#[async_trait::async_trait]
impl AsyncCallable for Rss {
    #[instrument(skip(self, ctx), ret)]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let url = as_string(&evaluated[0])?;
        let channel = Channel::from_str(&url)?;
        let items = channel
            .items()
            .into_iter()
            .map(TryFrom::try_from)
            .collect::<Result<Vec<_>>>()?;
        let feed = RssFeed {
            title: channel.title().to_string(),
            description: channel.description().to_string(),
            items,
        };
        let ret = serde_json::to_value(feed)?;
        Ok(ret.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::runtime::ScriptRuntime;

    #[tokio::test]
    async fn call_rss() {
        let mut runtime = ScriptRuntime::default();
        let template = serde_json::json!({
            "$eval": "rss(fetch('https://zenn.dev/feed'))"
        });
        let ret = runtime.run(&template, Default::default()).await;
        assert!(ret.is_ok());
    }
}
