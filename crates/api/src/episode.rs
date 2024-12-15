use crate::{client::ApiClient, trpc::TrpcClient};
use anyhow::Result;
use repos::entity::Episode;
use serde_json::json;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Section {
    Serif {
        speaker: String,
        text: String,
    },
    Audio {
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        from: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        to: Option<f64>,
    },
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewEpisode {
    pub podcast_id: String,
    pub title: String,
    pub description: Option<String>,
    pub sections: Vec<Section>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEpisode {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sections: Option<Vec<Section>>,
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
        self.mutation("newEpisode", serde_json::to_value(new_episode)?)
            .await?;
        Ok(())
    }

    pub async fn update_episode(&self, update_episode: UpdateEpisode) -> Result<()> {
        self.mutation("updateEpisode", serde_json::to_value(update_episode)?)
            .await?;
        Ok(())
    }
}
