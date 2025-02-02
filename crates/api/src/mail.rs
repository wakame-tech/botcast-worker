use crate::{client::ApiClient, trpc::TrpcClient};
use anyhow::Result;
use repos::entity::Mail;
use serde_json::json;

impl ApiClient {
    pub async fn mails(&self, corner_id: &str) -> Result<Vec<Mail>> {
        let resp = self
            .query(
                "mails",
                json!({
                    "cornerId": corner_id,
                }),
            )
            .await?;
        let mails: Vec<Mail> = serde_json::from_value(resp["mails"].clone())?;
        Ok(mails)
    }

    pub async fn new_mail(&self, corner_id: &str, body: serde_json::Value) -> Result<()> {
        self.mutation(
            "newMail",
            json!({
                "cornerId": corner_id,
                "body": body,
            }),
        )
        .await?;
        Ok(())
    }
}
