use crate::{
    trpc::{trpc_mutation, trpc_query},
    Credential,
};
use anyhow::Result;
use serde_json::{json, Value};

#[derive(Debug, serde::Deserialize)]
pub(crate) struct Script {
    id: String,
    title: String,
    template: Value,
    result: Value,
    user_id: String,
}

#[derive(Debug, serde::Serialize)]
struct UpdateScript {
    id: String,
    title: String,
    // json
    template: String,
}

#[derive(Debug, serde::Serialize)]
struct NewScript {
    title: String,
    // json
    template: String,
}

#[derive(Debug)]
pub(crate) struct ApiClient {
    endpoint: String,
    authorization: Option<String>,
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

    pub(crate) fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            authorization: None,
            client: reqwest::blocking::Client::new(),
        }
    }

    pub(crate) fn from_credential(credential: &Credential) -> Self {
        Self {
            endpoint: credential.endpoint.to_string(),
            authorization: Some(credential.token.clone()),
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
