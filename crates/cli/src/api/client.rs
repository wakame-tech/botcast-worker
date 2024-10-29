use super::{
    dto::{NewScript, Script, UpdateScript},
    trpc::{trpc_mutation, trpc_query},
};
use anyhow::Result;
use serde_json::{json, Value};

#[derive(Debug)]
pub(crate) struct ApiClient {
    endpoint: String,
    authorization: String,
    client: reqwest::Client,
}

impl ApiClient {
    async fn query(&self, name: &str, input: Value) -> Result<Value> {
        trpc_query(
            &self.client,
            &self.endpoint,
            name,
            input,
            self.authorization.clone(),
        )
        .await
    }

    async fn mutation(&self, name: &str, input: Value) -> Result<Value> {
        trpc_mutation(
            &self.client,
            &self.endpoint,
            name,
            input,
            self.authorization.clone(),
        )
        .await
    }

    pub(crate) fn new(endpoint: &str, token: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            authorization: token.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub(crate) async fn sign_in(&self, email: &str, password: &str) -> Result<String> {
        let resp = self
            .query(
                "signIn",
                json!({
                    "email": email,
                    "password": password,

                }),
            )
            .await?;
        let Value::String(token) = resp else {
            anyhow::bail!("{}", serde_json::to_string_pretty(&resp)?);
        };
        Ok(token)
    }

    pub(crate) async fn script(&self, id: &str) -> Result<Script> {
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

    pub(crate) async fn scripts(&self) -> Result<Vec<Script>> {
        let resp = self.query("scripts", json!({})).await?;
        let scripts: Vec<Script> = serde_json::from_value(resp["scripts"].clone())?;
        Ok(scripts)
    }

    pub(crate) async fn new_script(&self, input: NewScript) -> Result<Script> {
        let resp = self
            .mutation("newScript", serde_json::to_value(input)?)
            .await?;
        let script: Script = serde_json::from_value(resp["script"].clone())?;
        Ok(script)
    }

    pub(crate) async fn update_script(&self, input: UpdateScript) -> Result<()> {
        let resp = self
            .mutation("updateScript", serde_json::to_value(input)?)
            .await?;
        println!("{}", serde_json::to_string_pretty(&resp)?);
        Ok(())
    }
}
