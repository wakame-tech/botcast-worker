use crate::{
    imports::as_string,
    runtime::{display_fn_io, evaluate_args, ScriptRuntime},
};
use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Value},
    Context,
};
use script_llm::{chat_assistant, chat_completion, create_thread, delete_thread};

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
struct CreateThread {
    open_ai_api_key: String,
}

#[async_trait::async_trait]
impl AsyncCallable for CreateThread {
    async fn call(&self, _: &Context<'_>, args: &[Value]) -> Result<Value> {
        let ret = create_thread(self.open_ai_api_key.clone()).await?;
        let ret = serde_json::Value::String(ret);
        log::info!("{}", display_fn_io("create_thread", args, &ret)?);
        Ok(ret.into())
    }
}

#[derive(Clone)]
struct DeleteThread {
    open_ai_api_key: String,
}

#[async_trait::async_trait]
impl AsyncCallable for DeleteThread {
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let thread_id = as_string(&evaluated[0])?;
        delete_thread(self.open_ai_api_key.clone(), thread_id).await?;
        let ret = serde_json::Value::Null;
        log::info!("{}", display_fn_io("delete_thread", args, &ret)?);
        Ok(ret.into())
    }
}

#[derive(Clone)]
struct ChatAssistant {
    open_ai_api_key: String,
}

#[async_trait::async_trait]
impl AsyncCallable for ChatAssistant {
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let prompt = as_string(&evaluated[0])?;
        let assistant_id = as_string(&evaluated[1])?;
        let thread_id = as_string(&evaluated[2])?;
        let ret = chat_assistant(
            prompt,
            thread_id,
            assistant_id,
            self.open_ai_api_key.clone(),
        )
        .await?;
        let ret = serde_json::Value::String(ret);
        log::info!("{}", display_fn_io("llm_assistant", args, &ret)?);
        Ok(ret.into())
    }
}

pub fn register_llm_functions(runtime: &mut ScriptRuntime, open_ai_api_key: String) {
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
        }),
    );
    runtime.register_function(
        "create_thread",
        Box::new(CreateThread {
            open_ai_api_key: open_ai_api_key.clone(),
        }),
    );
    runtime.register_function(
        "delete_thread",
        Box::new(DeleteThread {
            open_ai_api_key: open_ai_api_key.clone(),
        }),
    );
}
