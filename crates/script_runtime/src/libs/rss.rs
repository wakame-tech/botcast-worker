use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use rss::{Channel, Item};

#[derive(Debug, serde::Serialize)]
pub(crate) struct RssFeed {
    title: String,
    description: String,
    items: Vec<RssFeedItem>,
}

impl TryFrom<Channel> for RssFeed {
    type Error = anyhow::Error;

    fn try_from(channel: Channel) -> Result<Self> {
        let items = channel
            .items()
            .iter()
            .map(TryFrom::try_from)
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            title: channel.title().to_string(),
            description: channel.description().to_string(),
            items,
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
