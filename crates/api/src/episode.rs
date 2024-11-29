use crate::{client::ApiClient, trpc::TrpcClient};
use anyhow::Result;
use repos::entity::Episode;
use serde_json::json;

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
}
