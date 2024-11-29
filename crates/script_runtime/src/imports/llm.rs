use crate::{
    imports::as_string,
    runtime::{evaluate_args, ScriptRuntime},
};
use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Value},
    Context,
};
use script_llm::{chat_assistant, chat_completion};
pub use script_llm::{create_thread, delete_thread};
use tracing::instrument;

#[derive(Clone)]
struct ChatCompletion {
    open_ai_api_key: String,
}

#[async_trait::async_trait]
impl AsyncCallable for ChatCompletion {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let prompt = as_string(&evaluated[0])?;
        let ret = chat_completion(prompt, self.open_ai_api_key.clone()).await?;
        tracing::info!("{}", ret);
        let ret = serde_json::Value::String(ret);
        Ok(ret.into())
    }
}

#[derive(Clone)]
struct ChatAssistant {
    open_ai_api_key: String,
    thread_id: String,
}

#[async_trait::async_trait]
impl AsyncCallable for ChatAssistant {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let prompt = as_string(&evaluated[0])?;
        let assistant_id = as_string(&evaluated[1])?;
        let ret = chat_assistant(
            prompt,
            self.thread_id.clone(),
            assistant_id,
            self.open_ai_api_key.clone(),
        )
        .await?;
        tracing::info!("{}", ret);
        let ret = serde_json::Value::String(ret);
        Ok(ret.into())
    }
}

pub fn register_llm_functions(
    runtime: &mut ScriptRuntime,
    open_ai_api_key: String,
    thread_id: String,
) {
    runtime.register_function(
        "llm",
        Box::new(ChatCompletion {
            open_ai_api_key: open_ai_api_key.clone(),
        }),
    );
    runtime.register_function(
        "llm_assistant",
        Box::new(ChatAssistant {
            open_ai_api_key: open_ai_api_key.clone(),
            thread_id,
        }),
    );
}
