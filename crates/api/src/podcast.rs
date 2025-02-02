use crate::{
    client::{ApiClient, User},
    trpc::TrpcClient,
};
use anyhow::Result;
use repos::entity::{Corner, Episode, Podcast};
use serde_json::json;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PodcastResponse {
    pub podcast: Podcast,
    pub episodes: Vec<Episode>,
    pub user: User,
    pub corners: Vec<Corner>,
}

impl PodcastResponse {
    fn from_trpc_response(res: serde_json::Value) -> Self {
        let podcast: Podcast = serde_json::from_value(res["podcast"].clone()).unwrap();
        let episodes: Vec<Episode> =
            serde_json::from_value(res["podcast"]["episodes"].clone()).unwrap();
        let user: User = serde_json::from_value(res["podcast"]["user"].clone()).unwrap();
        let corners: Vec<Corner> =
            serde_json::from_value(res["podcast"]["corners"].clone()).unwrap();
        Self {
            podcast,
            episodes,
            user,
            corners,
        }
    }
}

impl ApiClient {
    pub async fn podcast(&self, id: &str) -> Result<PodcastResponse> {
        let resp = self
            .query(
                "podcast",
                json!({
                    "id": id,
                }),
            )
            .await?;
        Ok(PodcastResponse::from_trpc_response(resp))
    }
}
