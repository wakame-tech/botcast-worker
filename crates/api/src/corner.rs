use crate::{client::ApiClient, trpc::TrpcClient};
use anyhow::Result;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewCorner {
    pub podcast_id: String,
    pub title: String,
    pub description: Option<String>,
    pub mail_schema: serde_json::Value,
}

impl ApiClient {
    pub async fn new_corner(&self, new_corner: NewCorner) -> Result<()> {
        self.query("newCorner", serde_json::to_value(new_corner)?)
            .await?;
        Ok(())
    }
}
