use crate::runtime::{display_fn_io, evaluate_args};
use anyhow::Result;
use json_e::{
    render_with_context,
    value::{AsyncCallable, Value},
    Context,
};
use script_llm::langchain;

#[derive(Clone)]
pub(crate) struct Llm;

#[async_trait::async_trait]
impl AsyncCallable for Llm {
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let prompt = match evaluated.as_slice() {
            [serde_json::Value::String(prompt)] => {
                render_with_context(&serde_json::Value::String(prompt.to_string()), ctx)
                    .await
                    .map_err(|e| anyhow::anyhow!("error: {}", e))
            }
            _ => Err(anyhow::anyhow!("llm only supports a string".to_string())),
        }?;
        let prompt = match prompt {
            serde_json::Value::String(prompt) => Ok(prompt),
            _ => Err(anyhow::anyhow!("llm only supports a string".to_string())),
        }?;
        let ret = langchain(&prompt)
            .await
            .map(serde_json::Value::String)
            .map_err(|e| anyhow::anyhow!("llm error: {}", e));
        log::info!("{}", display_fn_io("llm", args, &ret)?);
        Ok(ret?.into())
    }
}
