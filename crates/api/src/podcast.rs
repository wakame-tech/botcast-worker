use crate::{client::ApiClient, trpc::TrpcClient};
use anyhow::Result;
use repos::entity::Podcast;
use serde_json::json;

impl ApiClient {
    pub async fn podcast(&self, id: &str) -> Result<Podcast> {
        let resp = self
            .query(
                "podcast",
                json!({
                    "id": id,
                }),
            )
            .await?;
        let podcast: Podcast = serde_json::from_value(resp["podcast"].clone())?;
        Ok(podcast)
    }
}
