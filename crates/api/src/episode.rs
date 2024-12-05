use crate::{client::ApiClient, trpc::TrpcClient};
use anyhow::Result;
use repos::entity::Episode;
use serde_json::json;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Section {
    Serif { speaker: String, text: String },
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewEpisode {
    pub podcast_id: String,
    pub title: String,
    pub sections: Vec<Section>,
}

impl ApiClient {
    pub async fn episode(&self, id: &str) -> Result<Episode> {
        let resp = self
            .query(
                "episode",
                json!({
                    "id": id,
                }),
            )
            .await?;
        let episode: Episode = serde_json::from_value(resp["episode"].clone())?;
        Ok(episode)
    }

    pub async fn new_episode(&self, new_episode: NewEpisode) -> Result<()> {
        self.query("newEpisode", serde_json::to_value(new_episode)?)
            .await?;
        Ok(())
    }
}
