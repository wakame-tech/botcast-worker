use crate::{client::ApiClient, trpc::TrpcClient};
use anyhow::Result;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Script {
    pub id: String,
    pub title: String,
    pub template: Value,
    pub user_id: String,
}

#[derive(Debug, serde::Serialize)]
pub struct UpdateScript {
    pub id: Uuid,
    pub title: String,
    // json
    pub template: String,
}

#[derive(Debug, serde::Serialize)]
pub struct NewScript {
    pub title: String,
    // json
    pub template: String,
}

impl NewScript {
    pub fn new(title: String) -> Self {
        let default_template = serde_json::to_string(&json!({
            "$eval": "1+1"
        }))
        .unwrap();

        Self {
            title,
            template: default_template,
        }
    }
}

impl ApiClient {
    pub async fn script(&self, id: &str) -> Result<Script> {
        let resp = self
            .query(
                "script",
                json!({
                    "id": id,
                }),
            )
            .await?;
        let script: Script = serde_json::from_value(resp["script"].clone())?;
        Ok(script)
    }

    pub async fn scripts(&self) -> Result<Vec<Script>> {
        let resp = self.query("scripts", json!({})).await?;
        let scripts: Vec<Script> = serde_json::from_value(resp["scripts"].clone())?;
        Ok(scripts)
    }

    pub async fn new_script(&self, input: NewScript) -> Result<Script> {
        let resp = self
            .mutation("newScript", serde_json::to_value(input)?)
            .await?;
        let script: Script = serde_json::from_value(resp["script"].clone())?;
        Ok(script)
    }

    pub async fn update_script(&self, input: UpdateScript) -> Result<()> {
        let resp = self
            .mutation("updateScript", serde_json::to_value(input)?)
            .await?;
        println!("{}", serde_json::to_string_pretty(&resp)?);
        Ok(())
    }
}
