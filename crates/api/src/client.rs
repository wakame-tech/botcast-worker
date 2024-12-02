use crate::trpc::TrpcClient;
use anyhow::Result;
use serde_json::{json, Value};

#[derive(Debug)]
pub struct ApiClient {
    pub(crate) endpoint: String,
    pub(crate) authorization: String,
    pub(crate) client: reqwest::Client,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: String,
    pub auth_id: String,
    pub email: String,
    pub name: Option<String>,
}

impl ApiClient {
    pub fn new(endpoint: &str, token: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            authorization: token.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn me(&self) -> Result<User> {
        let resp = self.query("me", json!({})).await?;
        let user: User = serde_json::from_value(resp["user"].clone())?;
        Ok(user)
    }

    pub async fn sign_in(&self, email: &str, password: &str) -> Result<String> {
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
}
