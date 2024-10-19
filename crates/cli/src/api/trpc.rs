use anyhow::Result;
use reqwest::header::AUTHORIZATION;
use serde_json::Value;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct TrpcError {
    code: i32,
    message: String,
    data: Value,
}

pub(crate) fn trpc_query(
    client: &reqwest::blocking::Client,
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
        .send()?;
    let status = response.status();
    let body: Value = response.json()?;
    if status != 200 {
        let error: TrpcError = serde_json::from_value(body["error"].clone())?;
        return Err(anyhow::anyhow!("{}", serde_json::to_string_pretty(&error)?));
    }
    // println!("{}", serde_json::to_string_pretty(&body)?);
    Ok(body["result"]["data"].clone())
}

pub(crate) fn trpc_mutation(
    client: &reqwest::blocking::Client,
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
        .send()?;
    let status = response.status();
    let body: Value = response.json()?;
    if status != 200 {
        let error: TrpcError = serde_json::from_value(body["error"].clone())?;
        return Err(anyhow::anyhow!("{}", serde_json::to_string_pretty(&error)?));
    }
    // println!("{}", serde_json::to_string_pretty(&body)?);
    Ok(body["result"].clone())
}
