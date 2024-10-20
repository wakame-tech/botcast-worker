use crate::entity::{CommentId, EpisodeId, PodcastId, ScriptId};
use anyhow::Result;
use std::str::FromStr;
use uuid::Uuid;

/// triplet of ("urn", resource, resource_id)
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Urn {
    Podcast(PodcastId),
    Episode(EpisodeId),
    Comment(CommentId),
    Script(ScriptId),
    Other(String, Uuid),
}

impl FromStr for Urn {
    type Err = anyhow::Error;

    fn from_str(urn: &str) -> Result<Self> {
        let [sig, resource, resource_id]: [&str; 3] = urn
            .splitn(3, ':')
            .collect::<Vec<&str>>()
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid URN: {}", urn))?;
        anyhow::ensure!(sig == "urn", "Invalid URN: {}", urn);
        match resource {
            "podcast" => Ok(Urn::Podcast(PodcastId(resource_id.parse()?))),
            "episode" => Ok(Urn::Episode(EpisodeId(resource_id.parse()?))),
            "comment" => Ok(Urn::Comment(CommentId(resource_id.parse()?))),
            "script" => Ok(Urn::Script(ScriptId(resource_id.parse()?))),
            _ => Ok(Urn::Other(resource.to_string(), resource_id.parse()?)),
        }
    }
}
