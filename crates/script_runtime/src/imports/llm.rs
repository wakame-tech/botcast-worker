use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Value},
    Context,
};
use script_llm::langchain;

#[derive(Clone)]
pub(crate) struct Llm;

#[async_trait::async_trait]
impl AsyncCallable for Llm {
    async fn call(&self, _: &Context<'_>, args: &[Value]) -> Result<Value> {
        match args {
            [Value::String(prompt)] => Ok(Value::String(langchain(prompt).await?)),
            _ => Err(anyhow::anyhow!("llm only supports a string".to_string())),
        }
    }
}
