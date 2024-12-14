use anyhow::Result;
use std::iter::{self, empty};
use xq::module_loader::PreludeLoader;

pub(crate) fn run_xq(query: &str, value: serde_json::Value) -> Result<serde_json::Value> {
    let module_loader = PreludeLoader();
    let value: xq::Value = serde_json::from_value(value)
        .map_err(|_| anyhow::anyhow!("Failed to convert xq::Value"))?;
    let context = iter::once(Ok(value));
    let results = xq::run_query(query, context, empty(), &module_loader)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let mut values = vec![];
    for result in results {
        // dbg!(&result);
        let value: xq::Value =
            result.map_err(|e| anyhow::anyhow!("Failed to convert xq::Value: {:?}", e))?;
        let value: serde_json::Value = serde_json::to_value(value)?;
        values.push(value);
    }
    Ok(serde_json::Value::Array(values))
}
