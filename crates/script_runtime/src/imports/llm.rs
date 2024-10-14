use anyhow::Result;
use futures::{future::BoxFuture, FutureExt};
use json_e::{value::Value, Context};

pub fn llm<'a>(_: &Context<'_>, args: &'a [Value]) -> BoxFuture<'a, Result<Value>> {
    async move {
        match args {
            [Value::String(_prompt)] => {
                todo!();
            }
            _ => Err(anyhow::anyhow!("llm only supports a string".to_string())),
        }
    }
    .boxed()
}
