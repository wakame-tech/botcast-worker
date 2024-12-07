use crate::{
    imports::as_string,
    runtime::{evaluate_args, ScriptRuntime},
};
use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Value},
    Context,
};
use script_llm::{chat_assistant, chat_completion, function_calling};
pub use script_llm::{create_thread, delete_thread};
use tracing::instrument;

/// OpenAI ChatCompletion API
///
/// ```json
/// {
///     "$eval": "llm(api_key, prompt)"
/// }
/// ```
#[derive(Clone)]
struct ChatCompletion;

#[async_trait::async_trait]
impl AsyncCallable for ChatCompletion {
    #[instrument(skip(self, ctx), ret)]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let api_key = as_string(&evaluated[0])?;
        let prompt = as_string(&evaluated[1])?;

        let ret = chat_completion(api_key, prompt).await?;
        let ret = serde_json::Value::String(ret);
        Ok(ret.into())
    }
}

#[derive(Clone)]
struct FunctionCalling;

#[async_trait::async_trait]
impl AsyncCallable for FunctionCalling {
    #[instrument(skip(self, ctx), ret)]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let api_key = as_string(&evaluated[0])?;
        let prompt = as_string(&evaluated[1])?;
        let function = evaluated[2].clone();
        let ret = function_calling(api_key, prompt, function).await?;
        Ok(ret.into())
    }
}

///
/// ```json
/// {
///     "$let": {
///         "thread_id": {
///             "$eval": "create_thread(api_key)"
///          }
///     }
/// }
/// ```
#[derive(Clone)]
struct CreateThread;

#[async_trait::async_trait]
impl AsyncCallable for CreateThread {
    #[instrument(skip(self, ctx), ret)]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let api_key = as_string(&evaluated[0])?;

        let ret = create_thread(api_key).await?;
        let ret = serde_json::Value::String(ret);
        Ok(ret.into())
    }
}

///
/// ```json
/// {
///     "$eval": "delete_thread(api_key, thread_id)"
/// }
/// ```
#[derive(Clone)]
struct DeleteThread;

#[async_trait::async_trait]
impl AsyncCallable for DeleteThread {
    #[instrument(skip(self, ctx), ret)]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let api_key = as_string(&evaluated[0])?;
        let thread_id = as_string(&evaluated[1])?;

        delete_thread(api_key, thread_id).await?;
        Ok(Value::Null.into())
    }
}

/// OpenAI Assistant API
///
/// ```json
/// {
///     "$eval": "llm_assistant(api_key, thread_id, assistant_id, prompt)"
/// }
/// ```
#[derive(Clone)]
struct ChatAssistant;

#[async_trait::async_trait]
impl AsyncCallable for ChatAssistant {
    #[instrument(skip(self, ctx), ret)]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let api_key = as_string(&evaluated[0])?;
        let thread_id = as_string(&evaluated[1])?;
        let assistant_id = as_string(&evaluated[2])?;
        let prompt = as_string(&evaluated[3])?;

        let ret = chat_assistant(api_key, thread_id, assistant_id, prompt).await?;
        let ret = serde_json::Value::String(ret);
        Ok(ret.into())
    }
}

pub fn register_llm_functions(runtime: &mut ScriptRuntime) {
    runtime.register_function("create_thread", Box::new(CreateThread));
    runtime.register_function("delete_thread", Box::new(DeleteThread));
    runtime.register_function("llm", Box::new(ChatCompletion));
    runtime.register_function("llm_function_calling", Box::new(FunctionCalling));
    runtime.register_function("llm_assistant", Box::new(ChatAssistant));
}
