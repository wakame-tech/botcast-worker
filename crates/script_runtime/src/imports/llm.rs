use anyhow::Result;
use futures::{future::BoxFuture, FutureExt};
use json_e::{value::Value, Context};
use script_llm::langchain;

pub fn llm<'a>(_: &Context<'_>, args: &'a [Value]) -> BoxFuture<'a, Result<Value>> {
    async move {
        match args {
            [Value::String(prompt)] => Ok(Value::String(langchain(prompt).await?)),
            _ => Err(anyhow::anyhow!("llm only supports a string".to_string())),
        }
    }
    .boxed()
}
