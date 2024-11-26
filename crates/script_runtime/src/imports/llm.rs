use crate::{
    imports::as_string,
    runtime::{display_fn_io, evaluate_args, ScriptRuntime},
};
use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Value},
    Context,
};
use script_llm::{chat_assistant, chat_completion};
pub use script_llm::{create_thread, delete_thread};

#[derive(Clone)]
struct ChatCompletion {
    open_ai_api_key: String,
}

#[async_trait::async_trait]
impl AsyncCallable for ChatCompletion {
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let prompt = as_string(&evaluated[0])?;
        let ret = chat_completion(prompt, self.open_ai_api_key.clone()).await?;
        let ret = serde_json::Value::String(ret);
        log::info!("{}", display_fn_io("llm", args, &ret)?);
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
        let ret = serde_json::Value::String(ret);
        log::info!("{}", display_fn_io("llm_assistant", args, &ret)?);
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
