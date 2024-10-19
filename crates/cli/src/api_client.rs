use crate::{
    trpc::{trpc_mutation, trpc_query},
    Credential,
};
use anyhow::Result;
use serde_json::{json, Value};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct Script {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) template: Value,
    result: Value,
    user_id: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct UpdateScript {
    id: String,
    title: String,
    // json
    template: String,
}

impl UpdateScript {
    pub(crate) fn new(id: String, title: String, template: Value) -> Self {
        Self {
            id,
            title,
            template: serde_json::to_string(&template).unwrap(),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct NewScript {
    title: String,
    // json
    template: String,
}

impl NewScript {
    pub(crate) fn new(title: String) -> Self {
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
