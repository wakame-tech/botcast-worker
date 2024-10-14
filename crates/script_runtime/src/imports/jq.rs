use anyhow::Result;
use futures::{future::BoxFuture, FutureExt};
use json_e::{value::Value, Context};

pub fn jq<'a>(_: &Context<'_>, args: &'a [Value]) -> BoxFuture<'a, Result<Value>> {
    async move {
        match args {
            [value, Value::String(query)] => {
                let res = jq_rs::run(&query, &serde_json::to_string(value)?)?;
                Ok(Value::String(res))
            }
            _ => Err(anyhow::anyhow!("invalid arguments")),
        }
    }
    .boxed()
}
