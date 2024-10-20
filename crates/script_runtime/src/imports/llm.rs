use crate::imports::display_fn_io;
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
        let prompt = match args {
            [Value::String(prompt)] => Ok(prompt),
            _ => Err(anyhow::anyhow!("llm only supports a string".to_string())),
        }?;
        let ret = langchain(prompt)
            .await
            .map(serde_json::Value::String)
            .map_err(|e| anyhow::anyhow!("llm error: {}", e));
        log::info!("{}", display_fn_io("llm", args, &ret)?);
        Ok(ret?.into())
    }
}
