pub mod imports;
pub mod runtime;

use anyhow::Result;

/// triplet of ("urn", resource, resource_id)
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Urn(pub String);

pub fn parse_urn(urn: &Urn) -> Result<(String, String)> {
    let [sig, resource, resource_id]: [&str; 3] = urn
        .0
        .splitn(3, ':')
        .collect::<Vec<&str>>()
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid URN: {}", urn.0))?;
    anyhow::ensure!(sig == "urn", "Invalid URN: {}", urn.0);
    Ok((resource.to_string(), resource_id.to_string()))
}
