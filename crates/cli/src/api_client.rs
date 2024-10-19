use anyhow::Result;
use reqwest::header::AUTHORIZATION;
use serde_json::{json, Value};

#[derive(Debug, serde::Deserialize)]
struct Script {
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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TrpcError {
    code: i32,
    message: String,
    data: Value,
}

#[derive(Debug)]
pub(crate) struct ApiClient {
    base_url: String,
    authorization: Option<String>,
    client: reqwest::blocking::Client,
}

impl ApiClient {
    pub(crate) fn new(base_url: String) -> Self {
        Self {
            base_url,
            authorization: None,
            client: reqwest::blocking::Client::new(),
        }
    }

    pub(crate) fn new_with_auth(base_url: String, user: &str, pass: &str) -> Result<Self> {
        let mut client = Self::new(base_url);
        let token = client.sign_in(user, pass)?;
        client.authorization = Some(token);
        Ok(client)
    }

    fn trpc_query(&self, name: &str, input: Value) -> Result<Value> {
        let url = format!(
            "{}/trpc/{}?input={}",
            self.base_url,
            name,
            urlencoding::encode(&serde_json::to_string(&input)?)
        );
        let response = self
            .client
            .get(url)
            .header(
                AUTHORIZATION,
                self.authorization.clone().unwrap_or("".to_string()),
            )
            .send()?;
        let status = response.status();
        let body: Value = response.json()?;
        if status != 200 {
            let error: TrpcError = serde_json::from_value(body["error"].clone())?;
            return Err(anyhow::anyhow!("{}", serde_json::to_string_pretty(&error)?));
        }
        println!("{}", serde_json::to_string_pretty(&body)?);
        Ok(body["result"]["data"].clone())
    }

    fn trpc_mutation(&self, name: &str, input: Value) -> Result<Value> {
        let url = format!("{}/trpc/{}", self.base_url, name);
        let response = self
            .client
            .post(url)
            .json(&input)
            .header(
                AUTHORIZATION,
                self.authorization.clone().unwrap_or("".to_string()),
            )
            .send()?;
        let status = response.status();
        let body: Value = response.json()?;
        if status != 200 {
            let error: TrpcError = serde_json::from_value(body["error"].clone())?;
            return Err(anyhow::anyhow!("{}", serde_json::to_string_pretty(&error)?));
        }
        println!("{}", serde_json::to_string_pretty(&body)?);
        Ok(body["result"].clone())
    }

    fn sign_in(&self, email: &str, password: &str) -> Result<String> {
        let resp = self.trpc_query(
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

    fn script(&self, id: &str) -> Result<Script> {
        let resp = self.trpc_query(
            "script",
            json!({
                "id": id,
            }),
        )?;
        let script: Script = serde_json::from_value(resp["script"].clone())?;
        Ok(script)
    }

    fn new_script(&self, input: NewScript) -> Result<Script> {
        let resp = self.trpc_mutation("newScript", serde_json::to_value(input)?)?;
        let script: Script = serde_json::from_value(resp["script"].clone())?;
        Ok(script)
    }

    fn update_script(&self, input: UpdateScript) -> Result<()> {
        let resp = self.trpc_mutation("updateScript", serde_json::to_value(input)?)?;
        println!("{}", serde_json::to_string_pretty(&resp)?);
        Ok(())
    }
}
