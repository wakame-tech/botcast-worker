use crate::trpc::TrpcClient;
use anyhow::Result;
use serde_json::{json, Value};

#[derive(Debug)]
pub struct ApiClient {
    pub(crate) endpoint: String,
    pub(crate) authorization: String,
    pub(crate) client: reqwest::Client,
}

impl ApiClient {
    pub fn new(endpoint: &str, token: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            authorization: token.to_string(),
            client: reqwest::Client::new(),
        }
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
