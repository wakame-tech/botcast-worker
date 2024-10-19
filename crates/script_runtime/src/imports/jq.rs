use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Value},
    Context,
};
use std::iter::{self, empty};
use xq::module_loader::PreludeLoader;

fn run_xq(query: &str, value: serde_json::Value) -> Result<Value> {
    let module_loader = PreludeLoader();
    let value: serde_json::Value = value.try_into()?;
    let value: xq::Value = serde_json::from_value(value)
        .map_err(|_| anyhow::anyhow!("Failed to convert xq::Value"))?;
    let context = iter::once(Ok(value));
    let results = xq::run_query(&query, context, empty(), &module_loader)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let mut values = vec![];
    for result in results {
        dbg!(&result);
        let value: xq::Value =
            result.map_err(|e| anyhow::anyhow!("Failed to convert xq::Value: {:?}", e))?;
        let value: serde_json::Value = serde_json::to_value(value)
            .map_err(|_| anyhow::anyhow!("Failed to convert serde_json::Value"))?;
        let value: Value = value.try_into()?;
        values.push(value);
    }
    Ok(Value::Array(values))
}

#[derive(Clone)]
pub(crate) struct Jq;

#[async_trait::async_trait]
impl AsyncCallable for Jq {
    async fn call(&self, _: &Context<'_>, args: &[Value]) -> Result<Value> {
        match args {
            [value, Value::String(query)] => {
                let value: serde_json::Value = value.try_into()?;
                let result = run_xq(query, value)?;
                Ok(result)
            }
            _ => Err(anyhow::anyhow!("invalid arguments")),
        }
    }
}
