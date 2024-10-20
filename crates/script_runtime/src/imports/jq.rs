use crate::xq::run_xq;
use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Value},
    Context,
};

#[derive(Clone)]
pub(crate) struct Jq;

#[async_trait::async_trait]
impl AsyncCallable for Jq {
    async fn call(&self, _: &Context<'_>, args: &[Value]) -> Result<Value> {
        match args {
            [value, Value::String(query)] => {
                let value: serde_json::Value = value.try_into()?;
                let result = run_xq(query, value)?;
                Ok(result.into())
            }
            _ => Err(anyhow::anyhow!("invalid arguments")),
        }
    }
}
