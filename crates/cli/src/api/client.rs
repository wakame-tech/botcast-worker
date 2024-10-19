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
    client: reqwest::blocking::Client,
}

impl ApiClient {
    fn query(&self, name: &str, input: Value) -> Result<Value> {
        trpc_query(
            &self.client,
            &self.endpoint,
            name,
            input,
            self.authorization.clone(),
        )
    }

    fn mutation(&self, name: &str, input: Value) -> Result<Value> {
        trpc_mutation(
            &self.client,
            &self.endpoint,
            name,
            input,
            self.authorization.clone(),
        )
    }

    pub(crate) fn new(endpoint: &str, token: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            authorization: token.to_string(),
            client: reqwest::blocking::Client::new(),
        }
    }

    pub(crate) fn sign_in(&self, email: &str, password: &str) -> Result<String> {
        let resp = self.query(
            "signIn",
            json!({
                "email": email,
                "password": password,

            }),
        )?;
        let Value::String(token) = resp else {
            anyhow::bail!("{}", serde_json::to_string_pretty(&resp)?);
        };
        Ok(token)
    }

    pub(crate) fn script(&self, id: &str) -> Result<Script> {
        let resp = self.query(
            "script",
            json!({
                "id": id,
            }),
        )?;
        let script: Script = serde_json::from_value(resp["script"].clone())?;
        Ok(script)
    }

    pub(crate) fn scripts(&self) -> Result<Vec<Script>> {
        let resp = self.query("scripts", json!({}))?;
        let scripts: Vec<Script> = serde_json::from_value(resp["scripts"].clone())?;
        Ok(scripts)
    }

    pub(crate) fn new_script(&self, input: NewScript) -> Result<Script> {
        let resp = self.mutation("newScript", serde_json::to_value(input)?)?;
        let script: Script = serde_json::from_value(resp["script"].clone())?;
        Ok(script)
    }

    pub(crate) fn update_script(&self, input: UpdateScript) -> Result<()> {
        let resp = self.mutation("updateScript", serde_json::to_value(input)?)?;
        println!("{}", serde_json::to_string_pretty(&resp)?);
        Ok(())
    }
}
