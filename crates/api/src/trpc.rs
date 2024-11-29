use crate::client::ApiClient;
use anyhow::Result;
use reqwest::header::AUTHORIZATION;
use serde_json::Value;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct TrpcError {
    code: i32,
    message: String,
    data: Value,
}

async fn trpc_query(
    client: &reqwest::Client,
    endpoint: &str,
    name: &str,
    input: Value,
    authorization: String,
) -> Result<Value> {
    let url = format!(
        "{}/trpc/{}?input={}",
        endpoint,
        name,
        urlencoding::encode(&serde_json::to_string(&input)?)
    );
    let response = client
        .get(url)
        .header(AUTHORIZATION, authorization)
        .send()
        .await?;
    let status = response.status();
    let body: Value = response.json().await?;
    if status != 200 {
        let error: TrpcError = serde_json::from_value(body["error"].clone())?;
        return Err(anyhow::anyhow!("{}", serde_json::to_string_pretty(&error)?));
    }
    // println!("{}", serde_json::to_string_pretty(&body)?);
    Ok(body["result"]["data"].clone())
}

async fn trpc_mutation(
    client: &reqwest::Client,
    endpoint: &str,
    name: &str,
    input: Value,
    authorization: String,
) -> Result<Value> {
    let url = format!("{}/trpc/{}", endpoint, name);
    let response = client
        .post(url)
        .json(&input)
        .header(AUTHORIZATION, authorization)
        .send()
        .await?;
    let status = response.status();
    let body: Value = response.json().await?;
    if status != 200 {
        let error: TrpcError = serde_json::from_value(body["error"].clone())?;
        return Err(anyhow::anyhow!("{}", serde_json::to_string_pretty(&error)?));
    }
    // println!("{}", serde_json::to_string_pretty(&body)?);
    Ok(body["result"].clone())
}

#[async_trait::async_trait]
pub(crate) trait TrpcClient {
    async fn query(&self, name: &str, input: Value) -> Result<Value>;
    async fn mutation(&self, name: &str, input: Value) -> Result<Value>;
}

#[async_trait::async_trait]
impl TrpcClient for ApiClient {
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
}
