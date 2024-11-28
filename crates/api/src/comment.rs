use crate::{client::ApiClient, trpc::TrpcClient};
use anyhow::Result;
use repos::entity::Comment;
use serde_json::json;

impl ApiClient {
    pub async fn comment(&self, id: &str) -> Result<Comment> {
        let resp = self
            .query(
                "comment",
                json!({
                    "id": id,
                }),
            )
            .await?;
        let comment: Comment = serde_json::from_value(resp["comment"].clone())?;
        Ok(comment)
    }
}
